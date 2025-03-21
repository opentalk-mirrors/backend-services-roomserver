// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{convert::Infallible, fmt::Debug, sync::Arc};

use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use super::module_context::ModuleContext;
use crate::Settings;

/// The trait that defines a signaling module
///
/// Implementors can be added as a module to the room task. The room task will forward signaling events to the module
/// with the corresponding [`SignalingModule::NAMESPACE`]. All [`SignalingModule::on_event`] calls are handled in
/// sequence on the same task. Signaling modules are expected to spawn separate tasks when compute intense or
/// long-running operations need to be executed (See [`SignalingModule::Loopback`] for more details).
#[async_trait::async_trait]
pub trait SignalingModule: Send + Sized {
    /// The unique namespace for the module
    ///
    /// This is used as a general identifier to dispatch incoming signaling messages to the correct module.
    const NAMESPACE: ModuleId;

    /// The incoming websocket payload which is received as an [`SignalingEvent::WebsocketMessage`] in [`SignalingModule::on_event`]
    type Incoming: for<'de> Deserialize<'de> + Send;

    /// The outgoing websocket payload that is sent to the clients
    type Outgoing: Serialize + PartialEq + Debug + From<Self::Error> + Send;

    /// Internal result type for asynchronous tasks
    ///
    /// These are received as [`SignalingEvent::LoopbackMessage`] in the [`SignalingModule::on_event`] when an asynchronous
    /// task created by the module finishes.
    ///
    /// Tasks can be created with [`ModuleContext::spawn`] or [`ModuleContext::spawn_blocking`].
    type Loopback: Send + 'static;

    /// The non-fatal error that can be returned from [`SignalingModule::on_event`]
    ///
    /// Is converted into a websocket event and returned to the command issuing participant
    ///
    /// Use [`Infallible`] if there is no error case.
    type Error: ModuleError;

    /// Creates an instance of the interface to access the module
    async fn init(init_data: SignalingModuleInitData) -> Option<Self>;

    /// Receive the next event that was dispatched to this module.
    ///
    /// This function is performance critical.
    async fn on_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: SignalingEvent<Self>,
    ) -> Result<(), SignalingModuleError<Self::Error>>;

    /// Destroy the module and remove all associated resources
    ///
    /// Long running tasks must be spawned in a separate task
    async fn destroy(self) {}
}

/// The type received in [`SignalingModule::on_event`]
#[derive(Debug, Clone, Deserialize)]
pub enum SignalingEvent<M>
where
    M: SignalingModule,
{
    /// A websocket message was sent to the module
    WebsocketMessage {
        sender: ParticipantId,
        content: M::Incoming,
    },

    /// An asynchronous task which was started by the module completed
    LoopbackMessage(M::Loopback),
}

/// Data that a signaling module might require to initialize
#[derive(Clone, Debug)]
pub struct SignalingModuleInitData {
    /// The roomserver settings
    pub settings: Arc<Settings>,
}

/// Marker trait to allow us to convert the [`SignalingModule::Error`] into a [`SignalingModuleError`]
pub trait ModuleError: Debug + Send {}

impl ModuleError for Infallible {}

/// The error type returned by [`SignalingModule::on_event`]
#[derive(Debug)]
pub enum SignalingModuleError<E> {
    /// An non-fatal internal error occurred
    Internal(anyhow::Error),
    /// A fatal error occurred.
    ///
    /// This is considered to be unrecoverable, the module will be flagged as broken and deactivated
    Fatal(FatalError),
    /// The module specific error
    ///
    /// Is turned into a websocket message and returned to the command issuing participant
    Module(E),
}

impl<E> From<anyhow::Error> for SignalingModuleError<E> {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err)
    }
}

impl<E: ModuleError> From<E> for SignalingModuleError<E> {
    fn from(err: E) -> Self {
        Self::Module(err)
    }
}

#[derive(Debug)]
pub struct FatalError(pub anyhow::Error);

impl<E> From<FatalError> for SignalingModuleError<E> {
    fn from(err: FatalError) -> Self {
        Self::Fatal(err)
    }
}
