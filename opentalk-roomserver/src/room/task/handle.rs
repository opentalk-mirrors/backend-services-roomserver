// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use axum::extract::ws::WebSocket;
use opentalk_roomserver_types::room_parameters::RoomParameters;
use tokio::sync::{mpsc, oneshot};

use super::RoomTaskApiError;

#[derive(Debug, thiserror::Error)]
pub(crate) enum RoomTaskHandleError {
    /// The room task is gone.
    ///
    /// If the room task received the request, [`request`](RoomTaskHandleError::Gone::request) will be `None`.
    /// If the room task did not receive the request, it's returned to the sender by setting it here.
    #[error("The room task is no longer available")]
    Gone {
        /// If the request couldn't be dispatched to the room task, it's returned here.
        request: Option<Request>,
    },

    #[error("API request failed: {0}")]
    ApiError(#[from] RoomTaskApiError),
}

impl RoomTaskHandleError {
    /// Take the request from the error.
    ///
    /// If the message could not be delivered to the room task, the request
    /// might be required for error handling (e.g. closing the WebSocket).
    pub(crate) fn take_request(&mut self) -> Option<Request> {
        match self {
            RoomTaskHandleError::Gone { request } => request.take(),
            RoomTaskHandleError::ApiError(_) => None,
        }
    }
}

/// A handle for the a [`super::RoomTask`]
///
/// Is used for communication between the room task and the web server API
#[derive(Debug, Clone)]
pub(crate) struct RoomTaskHandle {
    pub(super) sender: mpsc::Sender<TaskMessage>,
}

impl RoomTaskHandle {
    async fn send_request(&self, request: Request) -> Result<(), RoomTaskHandleError> {
        let (tx, rx) = oneshot::channel();

        let msg = TaskMessage::new(request, tx);

        self.sender
            .send(msg)
            .await
            .map_err(|e| RoomTaskHandleError::Gone {
                request: Some(e.0.request),
            })?;

        rx.await
            .map_err(|_| RoomTaskHandleError::Gone { request: None })??;

        Ok(())
    }

    /// Refresh the room idle timeout to its original duration
    pub(crate) async fn refresh_idle_timeout(&self) -> Result<(), RoomTaskHandleError> {
        self.send_request(Request::RefreshIdleTimeout).await
    }

    /// Update the parameters for the room
    pub(crate) async fn update_parameter(
        &self,
        parameter: RoomParameters,
    ) -> Result<(), RoomTaskHandleError> {
        self.send_request(Request::UpdateParameter(parameter)).await
    }

    pub(crate) async fn accept_signaling_socket(
        &self,
        socket: WebSocket,
    ) -> Result<(), RoomTaskHandleError> {
        self.send_request(Request::WsJoin { socket }).await
    }
}

/// A message that can be send to a [`super::RoomTask`]
///
/// See [`Request`] for the possible messages.
pub(super) struct TaskMessage {
    /// The specific request
    pub request: Request,
    /// A channel for the [`RoomTask`](super::RoomTask) to respond
    pub response_channel: oneshot::Sender<Result<(), RoomTaskApiError>>,
}

impl TaskMessage {
    fn new(
        request: Request,
        response_channel: oneshot::Sender<Result<(), RoomTaskApiError>>,
    ) -> Self {
        Self {
            request,
            response_channel,
        }
    }
}

/// A request to a [`RoomTask`](super::RoomTask)
#[derive(Debug)]
pub(crate) enum Request {
    /// Refresh the room tasks idle timeout
    RefreshIdleTimeout,

    /// Update the parameters for the room
    UpdateParameter(RoomParameters),

    /// Join the room with a given websocket stream and sink
    WsJoin { socket: WebSocket },
}
