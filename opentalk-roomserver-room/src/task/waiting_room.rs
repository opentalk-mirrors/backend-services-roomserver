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
        let joined_at;
        let display_name = client_parameters.kind.display_name();
        let avatar_url = client_parameters.kind.avatar_url().map(str::to_string);
        let connection_ids = match self.waiting_participants.entry(participant_id) {
            Entry::Occupied(mut occupied_entry) => {
                // The user joins with a second device. The client parameter (e.g. username, role)
                // could have changed since the last connect. If the participant is
                // already connected we ignore the new client parameter.

                let state = occupied_entry.get_mut();
                state.connections.insert(connection_id, device_id);
                joined_at = state.joined_at;

                state.connections.keys().copied().collect()
            }
            Entry::Vacant(vacant_entry) => {
                joined_at = Utc::now();
                vacant_entry.insert(WaitingParticipant {
                    kind: client_parameters.kind,
                    role: client_parameters.role,
                    connections: HashMap::from_iter([(connection_id, device_id)]),
                    accepted: false,
                    joined_at,
                });

                vec![connection_id]
            }
        };

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
            CoreEvent::JoinedWaitingRoom {
                participant_id,
                connection_ids,
                joined_at,
                display_name,
                avatar_url,
            },
        )?;

        tracing::debug!("Participant entered waiting room");
        Ok(())
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub fn move_to_waiting_room(
        &mut self,
        participant_id: ParticipantId,
    ) -> Result<(), FatalError> {
        let Some(state) = self.participants.all_unfiltered.get_mut(&participant_id) else {
            tracing::error!("Failed to move unknown participant {participant_id} to waiting room");
            return Ok(());
        };

        let connections = mem::take(&mut state.connections);
        let connection_ids = connections.keys();
        self.message_router
            .move_to_waiting_room(connection_ids.clone());

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

        let connection_ids: Vec<ConnectionId> = connection_ids.copied().collect();
        let display_name = state.kind.display_name();
        let avatar_url = state.kind.avatar_url().map(str::to_owned);
        let room = state.room;

        for connection_id in &connection_ids {
            self.participant_disconnected(
                EventOrigin::Internal,
                participant_id,
                *connection_id,
                room,
                DisconnectReason::SentToWaitingRoom,
            )?;
        }

        self.message_router.conference.serialize_and_send(
            self.participants.connected().moderators().connection_ids(),
            CORE_MODULE_ID,
            None,
            CoreEvent::JoinedWaitingRoom {
                participant_id,
                connection_ids,
                joined_at: Utc::now(),
                display_name,
                avatar_url,
            },
        )?;

        Ok(())
    }
}
