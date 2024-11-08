// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use super::handle::{Request, Response, RoomTaskHandle, TaskMessage};
use super::idle_timeout::IdleTimeout;
use super::registry::RoomTaskRegistry;
use anyhow::Result;
use opentalk_roomserver_types::room_parameters::RoomParameters;
use opentalk_types_common::rooms::RoomId;
use tokio::sync::mpsc;

const TIMEOUT: u64 = 30;

/// The [`RoomTask`] manages the conference state and signaling.
///
/// An [`IdleTimeout`] starts when a room has no participants in it. When the idle timeout is reached, the room task
/// exits.
pub(super) struct RoomTask {
    /// the identifier of the room
    room_id: RoomId,
    /// The start parameters for the room task
    parameters: RoomParameters,
    /// The receiver for web server API request that target this room
    api_rx: mpsc::Receiver<TaskMessage>,
    /// The rooms idle timeout, only active when no participants are in the room.
    idle_timeout: IdleTimeout,
}

impl RoomTask {
    /// Spawns a new [`RoomTask`]
    pub(super) fn spawn(
        room_id: RoomId,
        room_parameters: RoomParameters,
        task_registry: RoomTaskRegistry,
    ) -> RoomTaskHandle {
        let (tx, rx) = mpsc::channel(20);

        let room_task = RoomTask {
            room_id,
            parameters: room_parameters,
            api_rx: rx,
            idle_timeout: IdleTimeout::start_new(TIMEOUT),
        };

        tokio::task::spawn(async move {
            room_task.run().await;
            task_registry.remove_room(room_id).await;
        });

        RoomTaskHandle { sender: tx }
    }

    async fn run(self) {
        if let Err(e) = self.inner_run().await {
            log::error!("RoomTask exited with error {}", e)
        }
    }

    async fn inner_run(mut self) -> Result<()> {
        // TODO: initialize modules

        // TODO: create ws listener

        loop {
            let rx = &mut self.api_rx;

            tokio::select! {
                msg = rx.recv() => {
                    let Some(msg) = msg else {
                        // TaskHandle dropped, exiting
                        log::warn!("Room tasks {} handle was dropped, exiting", self.room_id);
                        return Ok(());
                    };

                    self.handle_api_request(msg).await?;
                },
                _ = self.idle_timeout.has_timed_out() => {
                    log::debug!("Room task {} reached its idle timeout, exiting", self.room_id);
                    return Ok(());
                }
            };
        }
    }

    async fn handle_api_request(&mut self, msg: TaskMessage) -> Result<()> {
        match msg.request {
            Request::RefreshIdleTimeout => {
                self.idle_timeout.refresh(TIMEOUT);
                let _ = msg.response_channel.send(Response::Ack);
            }
            Request::UpdateParameter(room_parameters) => {
                self.parameters = room_parameters;
                // TODO: handle updated values
            }
        }

        Ok(())
    }
}
