// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_signaling::event_origin::ParticipantOrigin;
use opentalk_roomserver_types::{
    error::SignalingError,
    moderation::{
        MODERATION_MODULE_ID,
        command::{Accept, ModerationCommand},
        event::{ModerationError, ModerationEvent},
    },
    signaling::{SignalingCommand, module_error::SignalingModuleError},
};
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;

use crate::task::RoomTask;

impl<Socket: SignalingSocket> RoomTask<Socket> {
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
}
