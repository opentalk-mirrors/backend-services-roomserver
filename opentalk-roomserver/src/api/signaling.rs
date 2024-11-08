// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use axum::{
    async_trait,
    extract::ws::{close_code, CloseFrame, WebSocket},
};
use opentalk_roomserver_web_api::v1::signaling::SignalingBackend;
use opentalk_types::api::error::ApiError;
use opentalk_types_common::rooms::RoomId;

use super::Context;
use crate::room::task::{handle::RoomTaskHandleError, RoomTaskApiError};

impl From<RoomTaskHandleError> for ApiError {
    fn from(error: RoomTaskHandleError) -> Self {
        match error {
            RoomTaskHandleError::Gone => {
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
        mut socket: WebSocket,
        room_id: RoomId,
    ) -> Result<(), Self::Error> {
        let Some(task_handle) = self.room_tasks.get_task_handle(&room_id).await else {
            // the connection is probably already closed if this `send` fails
            let _ = socket
                .send(axum::extract::ws::Message::Close(Some(CloseFrame {
                    code: close_code::ERROR,
                    reason: "The room became unavailable.".into(),
                })))
                .await;
            return Err(ApiError::not_found());
        };

        task_handle.accept_signaling_socket(socket).await?;

        Ok(())
    }
}
