// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::{
    client_parameters::ClientParameters, room_parameters::RoomParameters,
};
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;
use opentalk_types_api_v1::error::ApiError;
use opentalk_types_common::users::UserId;
use tokio::sync::{
    mpsc,
    oneshot::{self, Receiver},
};
use tracing::Span;

use super::RoomTaskApiError;

#[derive(Debug, thiserror::Error)]
pub enum RoomTaskHandleError<Socket: SignalingSocket> {
    /// The room task is gone.
    ///
    /// If the room task received the request, [`request`](RoomTaskHandleError::Gone::request) will
    /// be `None`. If the room task did not receive the request, it's returned to the sender by
    /// setting it here.
    #[error("The room task is no longer available")]
    Gone {
        /// If the request couldn't be dispatched to the room task, it's returned here.
        request: Box<Option<Request<Socket>>>,
    },

    #[error("API request failed: {0}")]
    ApiError(#[from] RoomTaskApiError),
}

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

impl<Socket: SignalingSocket> RoomTaskHandleError<Socket> {
    /// Take the request from the error.
    ///
    /// If the message could not be delivered to the room task, the request
    /// might be required for error handling (e.g. closing the WebSocket).
    pub fn take_request(&mut self) -> Option<Request<Socket>> {
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
pub struct RoomTaskHandle<Socket: SignalingSocket> {
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
        let msg = TaskMessage::new(request);
        self.sender
            .send(msg)
            .await
            .map_err(|e| RoomTaskHandleError::Gone {
                request: Box::new(Some(e.0.request)),
            })?;

        Ok(())
    }

    async fn receive_response<T>(
        rx: Receiver<Result<T, RoomTaskApiError>>,
    ) -> Result<T, RoomTaskHandleError<Socket>> {
        let response = rx.await.map_err(|_| RoomTaskHandleError::Gone {
            request: Box::new(None),
        })??;

        Ok(response)
    }

    /// Refresh the room idle timeout to its original duration
    ///
    /// This can only fail if the room has reached its idle timeout or been removed by a user
    pub async fn refresh_idle_timeout(&self) -> Result<(), RoomTaskHandleError<Socket>> {
        let (tx, rx) = oneshot::channel();

        self.send_request(Request::RefreshIdleTimeout { response: tx })
            .await?;

        Self::receive_response(rx).await?;

        Ok(())
    }

    /// Update the parameters for the room
    pub async fn update_parameter(
        &self,
        parameter: RoomParameters,
    ) -> Result<(), RoomTaskHandleError<Socket>> {
        let (tx, rx) = oneshot::channel();

        self.send_request(Request::UpdateParameter {
            response: tx,
            parameters: Box::new(parameter),
        })
        .await?;

        Self::receive_response(rx).await?;

        Ok(())
    }

    pub async fn accept_signaling_socket(
        &self,
        socket: Socket,
        client_parameters: ClientParameters,
    ) -> Result<(), RoomTaskHandleError<Socket>> {
        let (tx, rx) = oneshot::channel();

        self.send_request(Request::WsJoin {
            response: tx,
            socket,
            client_parameters,
        })
        .await?;

        Self::receive_response(rx).await?;

        Ok(())
    }

    pub async fn is_banned(&self, user_id: UserId) -> Result<bool, RoomTaskHandleError<Socket>> {
        let (tx, rx) = oneshot::channel();

        self.send_request(Request::IsBanned {
            response: tx,
            user_id,
        })
        .await?;

        let response = Self::receive_response(rx).await?;

        Ok(response)
    }
}

/// A message that can be send to a [`super::RoomTask`]
///
/// See [`Request`] for the possible messages.
pub(super) struct TaskMessage<Socket: SignalingSocket> {
    /// The specific request
    pub request: Request<Socket>,
    /// Parent span on the caller side, used to track spans across channels.
    pub span: Span,
}

impl<Socket: SignalingSocket> TaskMessage<Socket> {
    fn new(request: Request<Socket>) -> Self {
        let span = tracing::Span::current();
        Self { request, span }
    }
}

type ResponseSender<T> = oneshot::Sender<Result<T, RoomTaskApiError>>;

/// A request to a [`RoomTask`](super::RoomTask)
#[derive(Debug)]
pub enum Request<Socket: SignalingSocket> {
    /// Refresh the room tasks idle timeout
    RefreshIdleTimeout { response: ResponseSender<()> },

    /// Update the parameters for the room
    UpdateParameter {
        response: ResponseSender<()>,
        parameters: Box<RoomParameters>,
    },

    /// Check if a user with the given [`UserId`] is banned
    IsBanned {
        response: ResponseSender<bool>,
        user_id: UserId,
    },

    /// Join the room with a given websocket stream and sink
    WsJoin {
        response: ResponseSender<()>,
        socket: Socket,
        client_parameters: ClientParameters,
    },
}
