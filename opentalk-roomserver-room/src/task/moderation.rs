// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{HashMap, hash_map::Entry};

use opentalk_roomserver_signaling::event_origin::ParticipantOrigin;
use opentalk_roomserver_types::{
    client_parameters::ClientParameters,
    connection_id::ConnectionId,
    device_id::DeviceId,
    error::SignalingError,
    moderation::{
        MODERATION_MODULE_ID,
        command::{Accept, ModerationCommand},
        event::{LeftWaitingRoom, ModerationError, ModerationEvent},
    },
    signaling::{
        SignalingCommand,
        module_error::{FatalError, SignalingModuleError},
    },
};
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;
use opentalk_types_signaling::ParticipantId;

use crate::task::RoomTask;

/// Information associated with a participant that joined the waiting room.
pub(super) struct WaitingParticipant {
    connections: HashMap<ConnectionId, DeviceId>,
    client_parameters: ClientParameters,
    accepted: bool,
}

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

        waiting.get_mut().connections.remove(&connection_id);
        if waiting.get().connections.is_empty() {
            waiting.remove();
        }

        let moderator_ids = self.participants.connected().moderators().connection_ids();

        let res = self
            .message_router
            .serialize_and_send(
                moderator_ids,
                MODERATION_MODULE_ID,
                None,
                ModerationEvent::LeftWaitingRoom(LeftWaitingRoom {
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
            .serialize_and_send(
                [connection_id],
                MODERATION_MODULE_ID,
                None,
                ModerationEvent::InWaitingRoom {
                    connection_id,
                    participant_id,
                },
            )
            .await?;

        let moderator_ids = self.participants.connected().moderators().connection_ids();

        self.message_router
            .serialize_and_send(
                moderator_ids,
                MODERATION_MODULE_ID,
                None,
                ModerationEvent::JoinedWaitingRoom { id: participant_id },
            )
            .await?;

        tracing::debug!("Participant entered waiting room");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn handle_moderation_command(
        &mut self,
        participant_origin: ParticipantOrigin,
        command: SignalingCommand,
    ) {
        tracing::trace!("received moderation command");

        let moderation_command: ModerationCommand =
            match serde_json::from_str(command.content.get()) {
                Ok(command) => command,
                Err(err) => {
                    tracing::debug!("invalid JSON");
                    self.message_router
                        .send_error(
                            participant_origin.connection_id,
                            participant_origin.transaction_id,
                            SignalingError::InvalidJson {
                                message: format!("{err:?}"),
                            },
                        )
                        .await;
                    return;
                }
            };

        let result = match moderation_command {
            ModerationCommand::Accept(accept) => {
                self.accept_waiting_room_participant(participant_origin, accept)
                    .await
            }
            ModerationCommand::EnterRoom => self.enter_room(participant_origin).await,
        };

        if let Err(e) = result {
            match e {
                SignalingModuleError::Internal(err) => {
                    tracing::error!("internal error in moderation module: {err:?}");

                    self.message_router
                        .send_error(
                            participant_origin.connection_id,
                            command.transaction_id,
                            SignalingError::Internal,
                        )
                        .await;
                }
                SignalingModuleError::Fatal(err) => {
                    tracing::error!("fatal error in moderation module: {err:?}");

                    self.message_router
                        .send_error(
                            participant_origin.connection_id,
                            command.transaction_id,
                            SignalingError::Internal,
                        )
                        .await;
                }
                SignalingModuleError::Module(module_error) => {
                    let result = self
                        .message_router
                        .serialize_and_send(
                            [participant_origin.connection_id],
                            MODERATION_MODULE_ID,
                            command.transaction_id,
                            ModerationEvent::Error(module_error),
                        )
                        .await;

                    if let Err(fatal_error) = result {
                        tracing::error!(
                            "failed to send error in moderation module: {fatal_error:?}"
                        );

                        self.message_router
                            .send_error(
                                participant_origin.connection_id,
                                command.transaction_id,
                                SignalingError::Internal,
                            )
                            .await;
                    }
                }
            };
        }
    }

    async fn accept_waiting_room_participant(
        &mut self,
        participant_origin: ParticipantOrigin,
        accept: Accept,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        let Some(moderator) = self.participants.all_unfiltered.get(&participant_origin.id) else {
            tracing::error!("Received moderation command from unknown participant");
            return Err(SignalingModuleError::Module(
                ModerationError::UnknownParticipant,
            ));
        };

        if !moderator.is_moderator() {
            tracing::debug!("Insufficient permissions");
            return Err(SignalingModuleError::Module(
                ModerationError::InsufficientPermissions,
            ));
        }

        let Some(participant) = self.waiting_participants.get_mut(&accept.target) else {
            tracing::debug!(
                "Failed to send `accept` to waiting participant: participant not known ({})",
                accept.target
            );
            return Err(SignalingModuleError::Module(ModerationError::NotWaiting));
        };

        participant.accepted = true;

        tracing::trace!("accept participant: {}", accept.target);
        self.message_router
            .serialize_and_send(
                participant.connections.keys().copied(),
                MODERATION_MODULE_ID,
                None,
                ModerationEvent::Accepted,
            )
            .await?;

        Ok(())
    }

    async fn enter_room(
        &mut self,
        participant_origin: ParticipantOrigin,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        let Entry::Occupied(participant) = self.waiting_participants.entry(participant_origin.id)
        else {
            tracing::debug!("Failed to enter room: participant not known");
            return Err(SignalingModuleError::Module(
                ModerationError::UnknownParticipant,
            ));
        };

        if !participant.get().accepted {
            tracing::debug!("Failed to enter room: participant not yet accepted");
            return Err(SignalingModuleError::Module(ModerationError::NotAccepted));
        }
        let participant = participant.remove();

        let moderator_ids = self.participants.connected().moderators().connection_ids();

        self.message_router
            .serialize_and_send(
                moderator_ids,
                MODERATION_MODULE_ID,
                None,
                ModerationEvent::LeftWaitingRoom(LeftWaitingRoom {
                    id: participant_origin.id,
                    connection_id: participant_origin.connection_id,
                }),
            )
            .await?;

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
