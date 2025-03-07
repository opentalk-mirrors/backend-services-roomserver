// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::fmt::Debug;

use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use super::module_context::ModuleContext;

/// The trait that defines a signaling module
///
/// Implementors can be added as a module to the room task. The room task will forward signaling events to the module
/// with the corresponding [`SignalingModule::NAMESPACE`]. All [`SignalingModule::on_event`] calls are handled in
/// sequence on the same task.
#[async_trait::async_trait]
pub trait SignalingModule: Send + Sized {
    /// The unique namespace for the module
    ///
    /// This is used as a general identifier to dispatch incoming signaling messages to the correct module.
    const NAMESPACE: ModuleId;

    /// The incoming websocket payload which is received as an [`SignalingEvent::WebsocketMessage`] in [`SignalingModule::on_event`]
    type Incoming: for<'de> Deserialize<'de> + Send;

    /// The outgoing websocket payload that is sent to the clients
    type Outgoing: Serialize + PartialEq + Debug + Send;

    /// Creates an instance of the interface to access the module
    async fn init() -> Option<Self>;

    /// Receive the next event that was dispatched to this module.
    ///
    /// This function is performance critical.
    async fn on_event(&mut self, ctx: &mut ModuleContext<'_, Self>, event: SignalingEvent<Self>);
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
}
