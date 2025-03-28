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
//! to remove any generic bounds. We achieve this with dynamic dispatch by storing them as a `Box<dyn ModuleDispatcher>`.
use std::{any::Any, collections::BTreeMap};

use anyhow::Context;
use module_context::{DynModuleContext, ModuleContext};
use opentalk_roomserver_types::error::SignalingError;
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::{ModuleData, ParticipantId};
use signaling_module::{FatalError, SignalingModule, SignalingModuleError};

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
        // TODO: make this owned
        ctx: &mut DynModuleContext<'_>,
        event: DynEvent,
    ) -> Result<(), FatalError>;

    async fn on_broadcast_event(
        &mut self,
        ctx: &mut DynModuleContext<'_>,
        event: &mut DynBroadcastEvent<'_>,
    ) -> Result<(), FatalError>;

    async fn destroy(self: Box<Self>);
}

pub enum DynEvent {
    WebsocketMessage {
        sender: ParticipantId,
        command: serde_json::Value,
    },
    LoopbackEvent(Box<dyn Any + Send + 'static>),
}

pub enum DynBroadcastEvent<'evt> {
    Joined {
        participant_id: ParticipantId,
        module_data: &'evt mut ModuleData,
        peer_module_data: &'evt mut BTreeMap<ParticipantId, BTreeMap<ModuleId, serde_json::Value>>,
    },
    Left(ParticipantId),
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
    async fn handle_event(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        event: DynEvent,
    ) -> Result<(), SignalingModuleError<M::Error>> {
        match event {
            DynEvent::WebsocketMessage { sender, command } => {
                self.handle_ws_event(ctx, sender, command).await
            }
            DynEvent::LoopbackEvent(result) => self.handle_loopback_event(ctx, result).await,
        }
    }

    async fn handle_broadcast_event(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        event: &mut DynBroadcastEvent<'_>,
    ) -> Result<(), SignalingModuleError<M::Error>> {
        match event {
            DynBroadcastEvent::Joined {
                participant_id,
                module_data,
                peer_module_data,
            } => {
                let join_info = self
                    .module
                    .on_participant_connected(ctx, *participant_id)
                    .await?;

                if let Some(success_info) = join_info.join_success {
                    module_data
                        .insert(&success_info)
                        .with_context(|| {
                            format!("failed to serialize JoinInfo for module '{}'", M::NAMESPACE)
                        })
                        .map_err(FatalError)?;
                }

                for (participant_id, peer_join_info) in join_info.peer.map {
                    let data = serde_json::to_value(peer_join_info)
                        .with_context(|| {
                            format!(
                                "failed to serialize PeerJoinInfo for module '{}'",
                                M::NAMESPACE
                            )
                        })
                        .map_err(FatalError)?;

                    peer_module_data
                        .entry(participant_id)
                        .or_default()
                        .insert(M::NAMESPACE, data);
                }
            }
            DynBroadcastEvent::Left(participant_id) => {
                self.module
                    .on_participant_disconnected(ctx, *participant_id)
                    .await?;
            }
        }
        Ok(())
    }

    /// Resolves a generic JSON message that was received by [`ModuleHandle::on_event`] to the concrete
    /// [`SignalingModule::Incoming`] type.
    async fn handle_ws_event(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        sender: ParticipantId,
        command: serde_json::Value,
    ) -> Result<(), SignalingModuleError<M::Error>> {
        let content = match serde_json::from_value(command) {
            Ok(content) => content,
            Err(err) => {
                log::debug!(
                    "failed to deserialize websocket message for namespace {}: {}",
                    M::NAMESPACE,
                    err
                );

                ctx.send_ws_error(SignalingError::InvalidJson {
                    message: "failed to deserialize websocket message".into(),
                })
                .await;

                return Ok(());
            }
        };

        self.module
            .on_websocket_message(ctx, sender, content)
            .await?;
        Ok(())
    }

    /// Resolves a dynamic loopback message that was received by [`ModuleHandle::on_event`] to the concrete
    /// [`SignalingModule::Loopback`] type.
    async fn handle_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        value: Box<dyn Any + Send + 'static>,
    ) -> Result<(), SignalingModuleError<M::Error>> {
        let event = value.downcast().ok().with_context(|| {
            format!(
                "Failed to downcast loopback type for module in namespace {}",
                M::NAMESPACE
            )
        })?;

        self.module.on_loopback_event(ctx, *event).await?;
        Ok(())
    }

    async fn handle_error(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        err: SignalingModuleError<M::Error>,
    ) -> Result<(), FatalError> {
        match err {
            SignalingModuleError::Fatal(err) => return Err(err),
            SignalingModuleError::Internal(err) => {
                log::error!(
                    "The '{}' module returned an internal error: {err:#?}",
                    M::NAMESPACE
                );
                ctx.send_ws_error(SignalingError::Internal).await;
            }
            SignalingModuleError::Module(err) => {
                let msg = err.into();

                ctx.send_ws_message(ctx.participant_id, msg).await?;
            }
        }
        Ok(())
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
        event: DynEvent,
    ) -> Result<(), FatalError> {
        let mut ctx = ModuleContext::<M>::new(ctx.reborrow());

        if let Err(err) = self.handle_event(&mut ctx, event).await {
            match err {
                SignalingModuleError::Fatal(err) => return Err(err),
                SignalingModuleError::Internal(err) => {
                    log::error!(
                        "The '{}' module returned an internal error: {err:#?}",
                        M::NAMESPACE
                    );
                    ctx.send_ws_error(SignalingError::Internal).await;
                }
                SignalingModuleError::Module(err) => {
                    let msg = err.into();

                    ctx.send_ws_message(ctx.participant_id, msg).await?;
                }
            }
        }

        Ok(())
    }

    async fn on_broadcast_event(
        &mut self,
        ctx: &mut DynModuleContext<'_>,
        event: &mut DynBroadcastEvent<'_>,
    ) -> Result<(), FatalError> {
        let mut ctx = ModuleContext::<M>::new(ctx.reborrow());

        if let Err(err) = self.handle_broadcast_event(&mut ctx, event).await {
            return self.handle_error(&mut ctx, err).await;
        }

        Ok(())
    }

    async fn destroy(self: Box<Self>) {
        self.module.destroy().await
    }
}
