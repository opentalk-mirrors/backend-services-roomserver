// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::Result;
use opentalk_roomserver_types::room_parameters::RoomParameters;
use tokio::sync::{mpsc, oneshot};

/// A handle for the a [`super::task::RoomTask`]
///
/// Is used for communication between the room task and the web server API
#[derive(Debug, Clone)]
pub(crate) struct RoomTaskHandle {
    pub(super) sender: mpsc::Sender<TaskMessage>,
}

impl RoomTaskHandle {
    async fn send_request(&self, request: Request) -> Result<Response> {
        let (tx, rx) = oneshot::channel();

        let msg = TaskMessage::new(request, tx);

        self.sender.send(msg).await?;

        let response = rx.await?;

        Ok(response)
    }

    /// Refresh the room idle timeout to its original duration
    pub(crate) async fn refresh_idle_timeout(&self) -> Result<()> {
        let response = self.send_request(Request::RefreshIdleTimeout).await?;

        match response {
            Response::Ack => Ok(()),
        }
    }

    /// Update the parameters for the room
    pub(crate) async fn update_parameter(&self, parameter: RoomParameters) -> Result<()> {
        let response = self
            .send_request(Request::UpdateParameter(parameter))
            .await?;

        match response {
            Response::Ack => Ok(()),
        }
    }
}

/// A message that can be send to a [`super::task::RoomTask`]
///
/// See [`Request`] and [`Response`] for the possible messages.
pub(super) struct TaskMessage {
    /// The specific request
    pub request: Request,
    /// A channel for the [`RoomTask`](super::task::RoomTask) to respond
    pub response_channel: oneshot::Sender<Response>,
}

impl TaskMessage {
    fn new(request: Request, response_channel: oneshot::Sender<Response>) -> Self {
        Self {
            request,
            response_channel,
        }
    }
}

/// A request to a [`RoomTask`](super::task::RoomTask)
#[derive(Debug, Clone)]
pub(crate) enum Request {
    /// Refresh the room tasks idle timeout
    RefreshIdleTimeout,

    /// Update the parameters for the room
    UpdateParameter(RoomParameters),
}

/// Responses that are sent from the [`RoomTask`](super::task::RoomTask)
#[derive(Debug, Clone)]
pub(crate) enum Response {
    /// Request was acknowledged
    Ack,
    // TODO: This would also contain an error type when a request can cause an error
}
