// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_signaling::{
    event_origin::{EventOrigin, ParticipantOrigin},
    internal_module_message::InterModuleMessage,
};
use opentalk_roomserver_types::{
    core::CORE_MODULE_ID,
    room_kind::RoomKind,
    signaling::{module_error::FatalError, websocket::SignalingSocket},
};
use opentalk_roomserver_types_livekit::{LIVEKIT_MODULE_ID, LiveKitInternal};
use opentalk_roomserver_types_subroom_audio::{
    SUBROOM_AUDIO_MODULE_ID, internal::SubroomAudioInternal,
};
use opentalk_roomserver_web_api::livekit_proxy::{
    LiveKitProxyTarget, WebsocketRequest, WebsocketResponse,
};
use opentalk_types_common::time::Timestamp;
use tokio::sync::oneshot::{self, Sender};

use crate::{RoomTaskApiError, task::RoomTask};

impl<Socket: SignalingSocket> RoomTask<Socket> {
    pub(super) fn send_socket_to_livekit_module(
        &mut self,
        websocket_request: WebsocketRequest,
        return_channel: Sender<Result<WebsocketResponse, RoomTaskApiError>>,
    ) -> Result<(), FatalError> {
        let (rx, tx) = oneshot::channel();

        let connection_id = websocket_request.connection_id;
        let participant_id = websocket_request.participant_id;

        match websocket_request.proxy_target.clone() {
            LiveKitProxyTarget::LiveKit { room_kind } => {
                self.handle_internal_command(
                    InterModuleMessage {
                        sender: CORE_MODULE_ID,
                        receiver: LIVEKIT_MODULE_ID,
                        command: Box::new(LiveKitInternal::ProxyLivekitSocket {
                            websocket_request: Box::new(websocket_request),
                            return_channel: rx,
                        }),
                    },
                    room_kind,
                    EventOrigin::Participant(ParticipantOrigin {
                        id: participant_id,
                        connection_id,
                        transaction_id: None,
                    }),
                    Timestamp::now(),
                )?;
            }
            LiveKitProxyTarget::SubroomAudio { .. } => {
                let Some(participant) = self
                    .participants
                    .all_unfiltered
                    .get(&websocket_request.participant_id)
                else {
                    return_channel
                        .send(Ok(WebsocketResponse::unauthorized()))
                        .ok();
                    return Ok(());
                };

                self.handle_internal_command(
                    InterModuleMessage {
                        sender: CORE_MODULE_ID,
                        receiver: SUBROOM_AUDIO_MODULE_ID,
                        command: Box::new(SubroomAudioInternal::ProxyLivekitSocket {
                            websocket_request: Box::new(websocket_request),
                            return_channel: rx,
                        }),
                    },
                    participant.room,
                    EventOrigin::Participant(ParticipantOrigin {
                        id: participant_id,
                        connection_id,
                        transaction_id: None,
                    }),
                    Timestamp::now(),
                )?;
            }
        }

        tokio::spawn(async {
            match tx.await {
                Ok(response) => return_channel.send(Ok(response)),
                Err(_) => return_channel.send(Ok(WebsocketResponse::internal_error())),
            }
        });
        Ok(())
    }

    pub(super) fn get_livekit_service_url(
        &mut self,
        return_channel: Sender<Result<String, RoomTaskApiError>>,
    ) -> Result<(), FatalError> {
        let (rx, tx) = oneshot::channel();

        self.handle_internal_command(
            InterModuleMessage {
                sender: CORE_MODULE_ID,
                receiver: LIVEKIT_MODULE_ID,
                command: Box::new(LiveKitInternal::GetLivekitServiceUrl { return_channel: rx }),
            },
            RoomKind::Main,
            EventOrigin::Internal,
            Timestamp::now(),
        )?;

        tokio::spawn(async {
            match tx.await {
                Ok(response) => return_channel.send(Ok(response)),
                Err(_) => return_channel.send(Err(RoomTaskApiError::NotFound)),
            }
        });
        Ok(())
    }
}
