// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! All [`SignalingModule`]s need to be accessed from the same collection by the [`RoomTask`](super::task::RoomTask).
//! They can not be stored directly in a single collection, because they use associated types and therefore are
//! generic. To store them in a single collection despite that, we make use of dynamic dispatch and polymorphism using
//! the [`ModuleHandle`] trait.
//!
//! The [`ModuleDispatcher`] acts as an intermediate between the generic JSON messages and the
//! [`SignalingModule`]s, turning received messages into concrete types that are specific to the [`SignalingModule`].
//! This works because the [`ModuleDispatcher`] is generic over the [`SignalingModule`] and can defer its
//! specific types.
//!
//! Due to the generic nature of the [`ModuleDispatcher`] they cannot be stored in a single collection either,
//! at least not when their generic type differs. This is where the [`ModuleHandle`] is used as an abstraction
//! to remove any generic bounds. We achieve this with dynamic dispatch by storing them as a [`Box<dyn ModuleCaller>`].
use std::any::Any;

use module_context::{DynModuleContext, ModuleContext};
use opentalk_roomserver_types::signaling::SignalingError;
use opentalk_types_signaling::ParticipantId;
use signaling_module::{SignalingEvent, SignalingModule};

pub mod module_context;
pub(crate) mod module_initializer;
pub(crate) mod ping;
pub mod signaling_module;

/// Abstracted handle to a [`SignalingModule`]
#[async_trait::async_trait]
pub trait ModuleHandle: Send {
    /// Invokes an event in the associated [`SignalingModule`]
    async fn on_event(&mut self, ctx: &mut DynModuleContext<'_>, event: DynEvent);
}

pub enum DynEvent {
    WebsocketMessage {
        sender: ParticipantId,
        command: serde_json::Value,
    },
    LoopbackEvent(Box<dyn Any + Send + 'static>),
}

/// Resolves generic JSON messages into concrete types for the associated [`SignalingModule`]
///
/// Implements the [`ModuleHandle`] trait.
struct ModuleDispatcher<M: SignalingModule + Send> {
    module: M,
}

impl<M> ModuleDispatcher<M>
where
    M: SignalingModule,
{
    /// Dispatches dynamic events to the correct modules and resolves their type
    async fn handle_event(&mut self, ctx: &mut ModuleContext<'_, M>, event: DynEvent) {
        match event {
            DynEvent::WebsocketMessage { sender, command } => {
                self.handle_ws_event(ctx, sender, command).await;
            }
            DynEvent::LoopbackEvent(result) => {
                self.handle_loopback_event(ctx, result).await;
            }
        }
    }

    /// Resolves a generic JSON message that was received by [`ModuleHandle::on_event`] to the concrete
    /// [`SignalingModule::Incoming`] type.
    async fn handle_ws_event(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        sender: ParticipantId,
        command: serde_json::Value,
    ) {
        let content = match serde_json::from_value(command) {
            Ok(content) => content,
            Err(err) => {
                log::debug!(
                    "failed to deserialize websocket message for namespace {}: {}",
                    M::NAMESPACE,
                    err
                );

                ctx.send_ws_error(
                    sender,
                    SignalingError::InvalidJson {
                        message: "failed to deserialize websocket message".into(),
                    },
                )
                .await;

                return;
            }
        };

        self.module
            .on_event(ctx, SignalingEvent::WebsocketMessage { sender, content })
            .await;
    }

    /// Resolves a dynamic loopback message that was received by [`ModuleHandle::on_event`] to the concrete
    /// [`SignalingModule::Loopback`] type.
    async fn handle_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        value: Box<dyn Any + Send + 'static>,
    ) {
        let Ok(value) = value.downcast() else {
            log::error!(
                "Failed to downcast loopback type for module in namespace {}",
                M::NAMESPACE
            );
            return;
        };

        self.module
            .on_event(ctx, SignalingEvent::LoopbackMessage(*value))
            .await;
    }
}

#[async_trait::async_trait]
impl<M> ModuleHandle for ModuleDispatcher<M>
where
    M: SignalingModule,
{
    async fn on_event(&mut self, ctx: &mut DynModuleContext<'_>, event: DynEvent) {
        let mut ctx = ModuleContext::<M>::new(ctx.reborrow());

        self.handle_event(&mut ctx, event).await;
    }
}
