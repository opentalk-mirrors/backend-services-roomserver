// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::room_parameters::RoomParameters;
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;
use tokio::sync::{mpsc, oneshot};

use super::RoomTaskApiError;

#[derive(Debug, thiserror::Error)]
pub(crate) enum RoomTaskHandleError<Socket: SignalingSocket> {
    /// The room task is gone.
    ///
    /// If the room task received the request, [`request`](RoomTaskHandleError::Gone::request) will be `None`.
    /// If the room task did not receive the request, it's returned to the sender by setting it here.
    #[error("The room task is no longer available")]
    Gone {
        /// If the request couldn't be dispatched to the room task, it's returned here.
        request: Option<Request<Socket>>,
    },

    #[error("API request failed: {0}")]
    ApiError(#[from] RoomTaskApiError),
}

impl<Socket: SignalingSocket> RoomTaskHandleError<Socket> {
    /// Take the request from the error.
    ///
    /// If the message could not be delivered to the room task, the request
    /// might be required for error handling (e.g. closing the WebSocket).
    pub(crate) fn take_request(&mut self) -> Option<Request<Socket>> {
        match self {
            RoomTaskHandleError::Gone { request } => request.take(),
            RoomTaskHandleError::ApiError(_) => None,
        }
    }
}

/// A handle for the a [`super::RoomTask`]
///
/// Is used for communication between the room task and the web server API
#[derive(Debug)]
pub(crate) struct RoomTaskHandle<Socket: SignalingSocket> {
    pub(super) sender: mpsc::Sender<TaskMessage<Socket>>,
}

// Manually implementing clone so that we don't require [`Socket`] to be
// Clone as well.
impl<Socket: SignalingSocket> Clone for RoomTaskHandle<Socket> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<Socket: SignalingSocket> RoomTaskHandle<Socket> {
    async fn send_request(
        &self,
        request: Request<Socket>,
    ) -> Result<(), RoomTaskHandleError<Socket>> {
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
    ///
    /// This can only fail if the room has reached its idle timeout or been removed by a user
    pub(crate) async fn refresh_idle_timeout(&self) -> Result<(), RoomTaskHandleError<Socket>> {
        self.send_request(Request::RefreshIdleTimeout).await
    }

    /// Update the parameters for the room
    pub(crate) async fn update_parameter(
        &self,
        parameter: RoomParameters,
    ) -> Result<(), RoomTaskHandleError<Socket>> {
        self.send_request(Request::UpdateParameter(parameter)).await
    }

    pub(crate) async fn accept_signaling_socket(
        &self,
        socket: Socket,
    ) -> Result<(), RoomTaskHandleError<Socket>> {
        self.send_request(Request::WsJoin { socket }).await
    }
}

/// A message that can be send to a [`super::RoomTask`]
///
/// See [`Request`] for the possible messages.
pub(super) struct TaskMessage<Socket: SignalingSocket> {
    /// The specific request
    pub request: Request<Socket>,
    /// A channel for the [`RoomTask`](super::RoomTask) to respond
    pub response_channel: oneshot::Sender<Result<(), RoomTaskApiError>>,
}

impl<Socket: SignalingSocket> TaskMessage<Socket> {
    fn new(
        request: Request<Socket>,
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
pub(crate) enum Request<Socket: SignalingSocket> {
    /// Refresh the room tasks idle timeout
    RefreshIdleTimeout,

    /// Update the parameters for the room
    UpdateParameter(RoomParameters),

    /// Join the room with a given websocket stream and sink
    WsJoin { socket: Socket },
}
