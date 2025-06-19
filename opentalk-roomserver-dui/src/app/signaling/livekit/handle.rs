// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use livekit::RoomEvent;
use opentalk_roomserver_client::api::event::Credentials;
use tokio::sync::{
    mpsc::{self, error::TryRecvError},
    watch,
};

#[derive(Debug)]
pub struct RunnerHandle {
    event_rx: mpsc::UnboundedReceiver<RoomEvent>,
    command_tx: mpsc::UnboundedSender<LiveKitRunnerCommand>,
    pub status_rx: watch::Receiver<Status>,
}

impl RunnerHandle {
    pub fn new(
        event_rx: mpsc::UnboundedReceiver<RoomEvent>,
        command_tx: mpsc::UnboundedSender<LiveKitRunnerCommand>,
        status_rx: watch::Receiver<Status>,
    ) -> Self {
        Self {
            event_rx,
            command_tx,
            status_rx,
        }
    }

    pub fn connect(&self, token: Credentials) -> anyhow::Result<()> {
        self.command_tx
            .send(LiveKitRunnerCommand::Connect { credentials: token })?;

        Ok(())
    }

    pub fn disconnect(&self) -> anyhow::Result<()> {
        self.command_tx.send(LiveKitRunnerCommand::Disconnect)?;
        Ok(())
    }

    pub fn recv_event(&mut self) -> anyhow::Result<Option<RoomEvent>> {
        match self.event_rx.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(TryRecvError::Disconnected) => anyhow::bail!("LiveKitRunner gone"),
            Err(TryRecvError::Empty) => Ok(None),
        }
    }
}

#[derive(Debug)]
pub enum Status {
    Disconnected,
    Connected,
}

impl Status {
    /// Returns `true` if the status is [`Disconnected`].
    ///
    /// [`Disconnected`]: Status::Disconnected
    #[must_use]
    pub fn is_disconnected(&self) -> bool {
        matches!(self, Self::Disconnected)
    }
}

#[derive(Debug)]
pub enum LiveKitRunnerCommand {
    Connect { credentials: Credentials },
    Disconnect,
}
