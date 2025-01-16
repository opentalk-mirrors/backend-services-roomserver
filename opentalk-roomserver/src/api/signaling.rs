// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use async_trait::async_trait;
use axum::extract::ws::{close_code, CloseFrame, WebSocket};
use opentalk_roomserver_web_api::v1::signaling::{websocket::SignalingSocket, SignalingBackend};
use opentalk_types_api_v1::error::ApiError;
use opentalk_types_common::rooms::RoomId;

use super::Context;
use crate::room::task::{
    handle::{Request, RoomTaskHandleError},
    RoomTaskApiError,
};

impl<Socket: SignalingSocket> From<RoomTaskHandleError<Socket>> for ApiError {
    fn from(error: RoomTaskHandleError<Socket>) -> Self {
        match error {
            RoomTaskHandleError::Gone { request: _ } => {
                Self::not_found().with_message("The requested room could not be found")
            }
            RoomTaskHandleError::ApiError(ref room_task_api_error) => match room_task_api_error {
                RoomTaskApiError::NotImplemented => {
                    ApiError::internal().with_message(error.to_string())
                }
            },
        }
    }
}

#[async_trait]
impl SignalingBackend for Context {
    type Error = ApiError;

    async fn ensure_room_exists(&self, room_id: RoomId) -> Result<(), Self::Error> {
        if self.room_tasks.ensure_room_exists(&room_id).await {
            Ok(())
        } else {
            Err(ApiError::not_found())
        }
    }

    async fn accept_client_stream(
        &self,
        socket: WebSocket,
        room_id: RoomId,
    ) -> Result<(), Self::Error> {
        let Some(task_handle) = self.room_tasks.get_task_handle(&room_id).await else {
            error_close_websocket(socket).await;
            return Err(ApiError::not_found());
        };

        let mut res = task_handle.accept_signaling_socket(socket).await;

        // handle that the socket might not reach the room task. In that case we need to close it ourself.
        if let Err(e) = &mut res {
            if let Some(Request::WsJoin { socket }) = e.take_request() {
                error_close_websocket(socket).await;
                return Err(ApiError::not_found());
            }
        }

        res?;
        Ok(())
    }
}

/// Closes the websocket because of an unexpected server error.
async fn error_close_websocket(mut socket: WebSocket) {
    // the connection is probably already closed if this `send` fails
    let _ = socket
        .send(axum::extract::ws::Message::Close(Some(CloseFrame {
            code: close_code::ERROR,
            reason: "The room became unavailable.".into(),
        })))
        .await;
}
