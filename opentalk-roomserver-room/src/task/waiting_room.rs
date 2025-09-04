// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{HashMap, hash_map::Entry},
    mem,
};

use chrono::Utc;
use opentalk_roomserver_signaling::{
    event_origin::EventOrigin, waiting_participant::WaitingParticipant,
};
use opentalk_roomserver_types::{
    client_parameters::ClientParameters,
    connection_id::ConnectionId,
    core::{CORE_MODULE_ID, CoreEvent, LeftWaitingRoom},
    device_id::DeviceId,
    disconnect_reason::DisconnectReason,
    signaling::module_error::FatalError,
};
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;
use opentalk_types_signaling::ParticipantId;

use crate::task::RoomTask;

impl<Socket: SignalingSocket> RoomTask<Socket> {
    #[tracing::instrument(level = "debug", skip(self))]
    pub(super) fn disconnect_waiting_participant(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) {
        let Entry::Occupied(mut waiting) = self.waiting_participants.entry(participant_id) else {
            tracing::error!("Attempted to disconnect waiting participant who does not exist");
            return;
        };

        self.message_router
            .waiting_room
            .remove_connection(connection_id);

        waiting.get_mut().connections.remove(&connection_id);
        if waiting.get().connections.is_empty() {
            waiting.remove();
        }

        let moderator_ids = self.participants.connected().moderators().connection_ids();

        let res = self.message_router.conference.serialize_and_send(
            moderator_ids,
            CORE_MODULE_ID,
            None,
            CoreEvent::LeftWaitingRoom(LeftWaitingRoom {
                id: participant_id,
                connection_id,
            }),
        );
        if let Err(e) = res {
            tracing::warn!("Failed to send disconnect message to moderator: {e:?}");
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn join_waiting_room(
        &mut self,
        connection_id: ConnectionId,
        participant_id: ParticipantId,
        device_id: DeviceId,
        client_parameters: ClientParameters,
    ) -> Result<(), FatalError> {
        match self.waiting_participants.entry(participant_id) {
            Entry::Occupied(mut occupied_entry) => {
                // The user joins with a second device. The client parameter (e.g. username, role)
                // could have changed since the last connect. If the participant is
                // already connected we ignore the new client parameter.

                occupied_entry
                    .get_mut()
                    .connections
                    .insert(connection_id, device_id);
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(WaitingParticipant {
                    kind: client_parameters.kind,
                    role: client_parameters.role,
                    connections: HashMap::from_iter([(connection_id, device_id)]),
                    accepted: false,
                    joined_at: Utc::now(),
                });
            }
        }

        self.message_router.waiting_room.serialize_and_send(
            [connection_id],
            CORE_MODULE_ID,
            None,
            CoreEvent::InWaitingRoom {
                connection_id,
                participant_id,
            },
        )?;

        let moderator_ids = self.participants.connected().moderators().connection_ids();

        self.message_router.conference.serialize_and_send(
            moderator_ids,
            CORE_MODULE_ID,
            None,
            CoreEvent::JoinedWaitingRoom { id: participant_id },
        )?;

        tracing::debug!("Participant entered waiting room");
        Ok(())
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub fn move_to_waiting_room(&mut self, participant_id: ParticipantId) {
        let Some(state) = self.participants.all_unfiltered.get_mut(&participant_id) else {
            tracing::error!("Failed to move unknown participant {participant_id} to waiting room");
            return;
        };

        let connections = mem::take(&mut state.connections);
        let ids = connections.keys();
        self.message_router.move_to_waiting_room(ids.clone());

        self.waiting_participants.insert(
            participant_id,
            WaitingParticipant {
                connections: connections.clone(),
                accepted: false,
                role: state.role,
                kind: state.kind.clone(),
                joined_at: Utc::now(),
            },
        );

        state.in_waiting_room = true;

        let room = state.room;
        for connection_id in ids {
            self.participant_disconnected(
                EventOrigin::Internal,
                participant_id,
                *connection_id,
                room,
                DisconnectReason::SentToWaitingRoom,
            );
        }
    }
}
