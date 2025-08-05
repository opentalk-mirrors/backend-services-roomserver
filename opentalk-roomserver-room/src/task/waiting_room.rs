// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{HashMap, hash_map::Entry};

use opentalk_roomserver_signaling::{
    event_origin::ParticipantOrigin, waiting_participant::WaitingParticipant,
};
use opentalk_roomserver_types::{
    client_parameters::ClientParameters,
    connection_id::ConnectionId,
    core::{CORE_MODULE_ID, CoreError, CoreEvent, LeftWaitingRoom},
    device_id::DeviceId,
    signaling::module_error::{FatalError, SignalingModuleError},
};
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;
use opentalk_types_signaling::ParticipantId;

use crate::task::RoomTask;

impl<Socket: SignalingSocket> RoomTask<Socket> {
    #[tracing::instrument(level = "debug", skip(self))]
    pub(super) async fn disconnect_waiting_participant(
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

        let res = self
            .message_router
            .conference
            .serialize_and_send(
                moderator_ids,
                CORE_MODULE_ID,
                None,
                CoreEvent::LeftWaitingRoom(LeftWaitingRoom {
                    id: participant_id,
                    connection_id,
                }),
            )
            .await;
        if let Err(e) = res {
            tracing::warn!("Failed to send disconnect message to moderator: {e:?}");
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn join_waiting_room(
        &mut self,
        connection_id: ConnectionId,
        participant_id: ParticipantId,
        device_id: DeviceId,
        client_parameters: ClientParameters,
    ) -> Result<(), FatalError> {
        match self.waiting_participants.entry(participant_id) {
            Entry::Occupied(mut occupied_entry) => {
                // The user joins with a second device. The client parameter (e.g. username, role) could have changed since the last connect.
                // If the participant is already connected we ignore the new client parameter.

                occupied_entry
                    .get_mut()
                    .connections
                    .insert(connection_id, device_id);
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(WaitingParticipant {
                    connections: HashMap::from([(connection_id, device_id)]),
                    client_parameters,
                    accepted: false,
                });
            }
        }

        self.message_router
            .waiting_room
            .serialize_and_send(
                [connection_id],
                CORE_MODULE_ID,
                None,
                CoreEvent::InWaitingRoom {
                    connection_id,
                    participant_id,
                },
            )
            .await?;

        let moderator_ids = self.participants.connected().moderators().connection_ids();

        self.message_router
            .conference
            .serialize_and_send(
                moderator_ids,
                CORE_MODULE_ID,
                None,
                CoreEvent::JoinedWaitingRoom { id: participant_id },
            )
            .await?;

        tracing::debug!("Participant entered waiting room");
        Ok(())
    }

    pub async fn enter_room(
        &mut self,
        participant_origin: ParticipantOrigin,
    ) -> Result<(), SignalingModuleError<CoreError>> {
        let Entry::Occupied(participant) = self.waiting_participants.entry(participant_origin.id)
        else {
            tracing::debug!("Failed to enter room: participant not known");
            return Err(CoreError::UnknownParticipant.into());
        };

        if !participant.get().accepted {
            tracing::debug!("Failed to enter room: participant not yet accepted");
            return Err(CoreError::NotAccepted.into());
        }
        let participant = participant.remove();

        let moderator_ids = self.participants.connected().moderators().connection_ids();

        self.message_router
            .conference
            .serialize_and_send(
                moderator_ids,
                CORE_MODULE_ID,
                None,
                CoreEvent::LeftWaitingRoom(LeftWaitingRoom {
                    id: participant_origin.id,
                    connection_id: participant_origin.connection_id,
                }),
            )
            .await?;

        self.message_router
            .upgrade_connections(participant.connections.keys());

        for (&connection_id, &device_id) in &participant.connections {
            self.join_room(
                participant_origin.id,
                connection_id,
                device_id,
                participant.client_parameters.clone(),
            )
            .await;
        }

        Ok(())
    }
}
