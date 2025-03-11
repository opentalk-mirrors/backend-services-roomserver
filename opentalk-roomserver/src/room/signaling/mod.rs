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
use anyhow::Result;
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
    async fn on_event(
        &mut self,
        ctx: &mut DynModuleContext<'_>,
        sender: ParticipantId,
        command: serde_json::Value,
    ) -> Result<()>;
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
    /// Resolves a generic JSON message that was received by [`ModuleHandle::on_event`] to the concrete
    /// [`SignalingModule::Incoming`] type.
    async fn handle_message(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        sender: ParticipantId,
        content: serde_json::Value,
    ) {
        let content = match serde_json::from_value(content) {
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
}

#[async_trait::async_trait]
impl<M> ModuleHandle for ModuleDispatcher<M>
where
    M: SignalingModule,
{
    async fn on_event(
        &mut self,
        ctx: &mut DynModuleContext<'_>,
        sender: ParticipantId,
        command: serde_json::Value,
    ) -> Result<()> {
        let mut ctx = ModuleContext::<M>::new(ctx.reborrow());

        self.handle_message(&mut ctx, sender, command).await;

        Ok(())
    }
}
