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
use std::{any::Any, cell::RefCell, collections::BTreeMap, time::Duration};

use anyhow::Context;
use dyn_module_context::DynModuleContext;
use opentalk_roomserver_signaling::{
    breakout::BreakoutRoom,
    module_context::ModuleContext,
    signaling_module::{
        CreateReplica, FatalError, SharedRawJson, SignalingModule, SignalingModuleError,
    },
};
use opentalk_roomserver_types::{
    breakout_id::BreakoutId, connection_id::ConnectionId, error::SignalingError,
};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::{ModuleData, ParticipantId};
use serde_json::value::RawValue;

pub mod dyn_module_context;
pub(crate) mod module_initializer;

/// Abstracted handle to a [`SignalingModule`]
#[async_trait::async_trait]
pub trait ModuleHandle: Send + Sync {
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
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        command: Box<RawValue>,
    },
    LoopbackEvent(Box<dyn Any + Send + 'static>),
}

pub enum DynBroadcastEvent<'evt> {
    Connected {
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        module_data: &'evt mut ModuleData,
        peer_module_data: &'evt mut BTreeMap<ParticipantId, BTreeMap<ModuleId, SharedRawJson>>,
    },

    Disconnected {
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    },

    BreakoutStart {
        rooms: &'evt Vec<BreakoutRoom>,
        duration: Option<Duration>,
    },

    BreakoutStop,

    /// A participant switches between main and/or breakout rooms
    SwitchRoom {
        participant_id: ParticipantId,
        old_room: Option<BreakoutId>,
        new_room: Option<BreakoutId>,
        /// The module data for the participant in the new room. Each connection needs to have their own module data
        module_data: &'evt mut BTreeMap<ConnectionId, ModuleData>,
    },
}

/// Resolves generic JSON messages into concrete types for the associated [`SignalingModule`]
///
/// Implements the [`ModuleHandle`] trait.
pub struct ModuleDispatcher<M: SignalingModule + Send> {
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
            DynEvent::WebsocketMessage {
                participant_id: sender,
                connection_id,
                command,
            } => {
                self.handle_ws_event(ctx, sender, connection_id, command)
                    .await
            }
            DynEvent::LoopbackEvent(result) => self.handle_loopback_event(ctx, result).await,
        }
    }

    #[tracing::instrument(skip_all, fields(opentalk.module = %M::NAMESPACE))]
    async fn handle_broadcast_event(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        event: &mut DynBroadcastEvent<'_>,
    ) -> Result<(), SignalingModuleError<M::Error>> {
        match event {
            DynBroadcastEvent::Connected {
                participant_id,
                connection_id,
                module_data,
                peer_module_data,
            } => {
                let is_first_connection = ctx
                    .participants
                    .get(participant_id)
                    .map(|s| s.connections.is_empty())
                    .unwrap_or(true);

                let join_info = self.module.on_participant_joined(
                    ctx,
                    *participant_id,
                    *connection_id,
                    is_first_connection,
                )?;

                if let Some(success_info) = join_info.join_success {
                    module_data
                        .insert(&success_info)
                        .with_context(|| {
                            format!("failed to serialize JoinInfo for module '{}'", M::NAMESPACE)
                        })
                        .map_err(FatalError)?;
                }

                for (participant_id, peer_join_info) in join_info.peer.map {
                    peer_module_data
                        .entry(participant_id)
                        .or_default()
                        .insert(M::NAMESPACE, peer_join_info);
                }
            }
            DynBroadcastEvent::Disconnected {
                participant_id,
                connection_id,
            } => {
                self.module
                    .on_participant_disconnected(ctx, *participant_id, *connection_id)?;
            }
            DynBroadcastEvent::BreakoutStart { rooms, duration } => {
                self.module.on_breakout_start(ctx, rooms, *duration)?;
            }

            DynBroadcastEvent::SwitchRoom {
                participant_id,
                old_room,
                new_room,
                module_data,
            } => {
                let join_infos =
                    self.module
                        .on_breakout_switch(ctx, *participant_id, *old_room, *new_room)?;

                for (conn_id, join_info) in join_infos {
                    module_data
                        .entry(conn_id)
                        .or_default()
                        .insert(&join_info)
                        .with_context(|| {
                            format!("failed to serialize JoinInfo for module '{}'", M::NAMESPACE)
                        })
                        .map_err(FatalError)?;
                }
            }

            DynBroadcastEvent::BreakoutStop => self.module.on_breakout_stop(ctx)?,
        }
        Ok(())
    }

    /// Resolves a generic JSON message that was received by [`ModuleHandle::on_event`] to the concrete
    /// [`SignalingModule::Incoming`] type.
    #[tracing::instrument(skip_all, fields(opentalk.command.sender = %sender, opentalk.module = %M::NAMESPACE))]
    async fn handle_ws_event(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        sender: ParticipantId,
        connection_id: ConnectionId,
        command: Box<RawValue>,
    ) -> Result<(), SignalingModuleError<M::Error>> {
        let content: <M as SignalingModule>::Incoming = match serde_json::from_str(command.get()) {
            Ok(content) => content,
            Err(err) => {
                log::debug!(
                    "failed to deserialize websocket message for namespace {}: {}",
                    M::NAMESPACE,
                    err
                );

                ctx.send_ws_error(SignalingError::InvalidJson {
                    message: "failed to deserialize websocket message".into(),
                });

                return Ok(());
            }
        };

        if let Some(replication_event) = content.replicate() {
            ctx.send_replica(sender, connection_id, replication_event)?;
        }

        self.module
            .on_websocket_message(ctx, sender, connection_id, content)?;

        Ok(())
    }

    /// Resolves a dynamic loopback message that was received by [`ModuleHandle::on_event`] to the concrete
    /// [`SignalingModule::Loopback`] type.
    #[tracing::instrument(skip_all, fields(opentalk.module = %M::NAMESPACE))]
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

        self.module.on_loopback_event(ctx, *event)?;
        Ok(())
    }

    #[tracing::instrument(skip_all, fields(opentalk.module = %M::NAMESPACE))]
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
                ctx.send_ws_error(SignalingError::Internal);
            }
            SignalingModuleError::Module(err) => {
                let msg = err.into();

                ctx.send_ws_message([ctx.participant_id], msg)?;
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
        let mut messages = RefCell::new(Vec::new());
        let mut module_context = ctx.reborrow().into_typed_context(&mut messages);

        if let Err(err) = self.handle_event(&mut module_context, event).await {
            match err {
                SignalingModuleError::Fatal(err) => return Err(err),
                SignalingModuleError::Internal(err) => {
                    log::error!(
                        "The '{}' module returned an internal error: {err:#?}",
                        M::NAMESPACE
                    );
                    module_context.send_ws_error(SignalingError::Internal);
                }
                SignalingModuleError::Module(err) => {
                    let msg = err.into();

                    module_context.send_ws_message([module_context.participant_id], msg)?;
                }
            }
        }

        for (connection_id, message) in messages.into_inner() {
            ctx.message_router
                .send_event([connection_id], message)
                .await;
        }

        Ok(())
    }

    async fn on_broadcast_event(
        &mut self,
        ctx: &mut DynModuleContext<'_>,
        event: &mut DynBroadcastEvent<'_>,
    ) -> Result<(), FatalError> {
        let mut messages = RefCell::new(Vec::new());
        let mut module_context: ModuleContext<'_, M> =
            ctx.reborrow().into_typed_context(&mut messages);

        if let Err(err) = self
            .handle_broadcast_event(&mut module_context, event)
            .await
        {
            return self.handle_error(&mut module_context, err).await;
        }

        for (connection_id, message) in messages.into_inner() {
            ctx.message_router
                .send_event([connection_id], message)
                .await;
        }

        Ok(())
    }

    async fn destroy(self: Box<Self>) {
        self.module.destroy()
    }
}
