// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::BTreeMap, time::Duration};

use anyhow::{Context, anyhow};
use chrono::TimeDelta;
use opentalk_roomserver_signaling::{
    event_origin::{EventOrigin, ParticipantOrigin},
    participant_state::ParticipantState,
};
use opentalk_roomserver_types::{
    breakout::{
        BREAKOUT_MODULE_ID, BreakoutRoom,
        breakout_config::BreakoutConfig,
        breakout_id::BreakoutId,
        command::BreakoutCommand,
        event::{BreakoutError, BreakoutEvent},
        module_data::{BreakoutModuleData, BreakoutPeerModuleData},
    },
    connection_id::ConnectionId,
    error::SignalingError,
    room_kind::RoomKind,
    signaling::{SignalingCommand, module_error::SignalingModuleError},
};
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;
use opentalk_types_common::time::Timestamp;
use opentalk_types_signaling::{ModuleData, ParticipantId};
use state::BreakoutState;
use tracing::Instrument;

use super::RoomTask;
use crate::signaling::DynBroadcastEvent;

pub const MIN_BREAKOUT_DURATION: Duration = Duration::from_secs(60);
pub const MAX_BREAKOUT_STOP_DELAY: Duration = Duration::from_secs(86400); // 1 day

pub(crate) mod state;

impl<Socket: SignalingSocket> RoomTask<Socket> {
    /// Serialize and handle the breakout [`SignalingCommand`]
    ///
    /// Similar to the `core` namespace, the breakout commands are not handled by a designated signaling module but
    /// directly by the [`RoomTask`]. Any fatal errors that occur here will be considered internal errors.
    pub(crate) async fn handle_breakout_command(
        &mut self,
        participant_origin: ParticipantOrigin,
        command: SignalingCommand,
    ) {
        let Some(participant_state) = self.participants.all_unfiltered.get(&participant_origin.id)
        else {
            tracing::error!(
                "failed to get participant state for participant `{}`",
                participant_origin.id
            );

            // This scenario should never occur because we never delete known participants. We still attempt to
            // send an error to the non-existent connection in a best-effort approach.
            self.message_router
                .conference
                .send_error(
                    participant_origin.connection_id,
                    command.transaction_id,
                    SignalingError::Internal,
                )
                .await;

            return;
        };

        let room_scope = participant_state.room;

        let breakout_command = match serde_json::from_str(command.payload.get()) {
            Ok(breakout_command) => breakout_command,
            Err(err) => {
                self.message_router
                    .conference
                    .send_error(
                        participant_origin.connection_id,
                        participant_origin.transaction_id,
                        SignalingError::InvalidJson {
                            message: format!("{err:?}"),
                        },
                    )
                    .await;

                return;
            }
        };

        let result = match breakout_command {
            BreakoutCommand::Start(config) => {
                self.breakout_start(participant_origin, room_scope, config)
                    .await
            }
            BreakoutCommand::SwitchRoom(new_room) => {
                self.switch_room(participant_origin.id, new_room, participant_origin.into())
                    .await
            }
            BreakoutCommand::Stop { delay } => self.breakout_stop(participant_origin, delay).await,
        };

        if let Err(e) = result {
            match e {
                SignalingModuleError::Internal(err) => {
                    tracing::error!("internal error in breakout module: {err:?}");

                    self.message_router
                        .conference
                        .send_error(
                            participant_origin.connection_id,
                            command.transaction_id,
                            SignalingError::Internal,
                        )
                        .await;
                }
                SignalingModuleError::Fatal(err) => {
                    tracing::error!("fatal error in breakout module: {err:?}");

                    self.message_router
                        .conference
                        .send_error(
                            participant_origin.connection_id,
                            command.transaction_id,
                            SignalingError::Internal,
                        )
                        .await;
                }
                SignalingModuleError::Module(module_error) => {
                    let result = self
                        .message_router
                        .conference
                        .serialize_and_send(
                            [participant_origin.connection_id],
                            BREAKOUT_MODULE_ID,
                            command.transaction_id,
                            BreakoutEvent::Error(module_error),
                        )
                        .await;

                    if let Err(fatal_error) = result {
                        tracing::error!("failed to send error in breakout module: {fatal_error:?}");

                        self.message_router
                            .conference
                            .send_error(
                                participant_origin.connection_id,
                                command.transaction_id,
                                SignalingError::Internal,
                            )
                            .await;
                    }
                }
            };
        }
    }

    /// Start the breakout rooms
    async fn breakout_start(
        &mut self,
        participant_origin: ParticipantOrigin,
        room_scope: RoomKind,
        config: BreakoutConfig,
    ) -> Result<(), SignalingModuleError<BreakoutError>> {
        let participants_state = self
            .participants
            .all_unfiltered
            .get(&participant_origin.id)
            .context("received message from unknown participant")?;

        if !participants_state.is_moderator() {
            return Err(BreakoutError::InsufficientPermission.into());
        }

        if self.breakout_config.is_some() {
            return Err(BreakoutError::AlreadyActive.into());
        }

        let mut assignments = BTreeMap::new();
        let mut breakout_rooms = Vec::with_capacity(config.rooms.len());
        let breakout_duration = config.duration.map(|d| d.max(MIN_BREAKOUT_DURATION));

        for (id, parameter) in config.rooms.iter().enumerate() {
            let breakout_id = BreakoutId::from(id as u64);

            for participant_id in &parameter.assignments {
                if !self
                    .participants
                    .all_unfiltered
                    .contains_key(participant_id)
                {
                    return Err(BreakoutError::UnknownParticipant {
                        participant_id: *participant_id,
                    }
                    .into());
                };

                if assignments.insert(*participant_id, breakout_id).is_some() {
                    return Err(BreakoutError::InvalidSelection.into());
                }
            }

            breakout_rooms.push(BreakoutRoom {
                id: breakout_id,
                name: parameter.name.clone(),
            });
        }
        let breakout_started = DynBroadcastEvent::BreakoutStart {
            rooms: &breakout_rooms,
            duration: breakout_duration,
        };

        let breakout_config = BreakoutState::init(config);
        let expires_at = breakout_config.expires_at;

        self.breakout_config = Some(breakout_config);

        let actions = self.broadcast_event_to_modules(
            EventOrigin::Participant(participant_origin),
            room_scope,
            breakout_started,
        );

        for (p, state) in self.participants.connected().iter() {
            let breakout_started = BreakoutEvent::Started {
                started_by: participant_origin.id,
                rooms: breakout_rooms.clone(),
                expires_at,
                assignment: assignments.get(p).copied(),
            };

            self.message_router
                .conference
                .serialize_and_send(
                    state.connections(),
                    BREAKOUT_MODULE_ID,
                    participant_origin.transaction_id,
                    breakout_started,
                )
                .await?;
        }

        actions.handle_requested_messages(self).await;

        Ok(())
    }

    /// Stop the breakout rooms and move participants back to the main room
    ///
    /// When a `delay` is provided, this will send out a [`BreakoutEvent::CloseNotice`] and keep the breakout rooms
    /// alive for the specified duration before closing them
    async fn breakout_stop(
        &mut self,
        origin: ParticipantOrigin,
        delay: Option<Duration>,
    ) -> Result<(), SignalingModuleError<BreakoutError>> {
        let participants_state = self
            .participants
            .all_unfiltered
            .get(&origin.id)
            .context("received message from unknown participant")?;

        if !participants_state.is_moderator() {
            return Err(BreakoutError::InsufficientPermission.into());
        }

        if self.breakout_config.is_none() {
            return Err(BreakoutError::BreakoutInactive.into());
        };

        let mut delay = match delay {
            Some(Duration::ZERO) | None => {
                self.close_breakout_rooms(origin.into()).await?;
                return Ok(());
            }
            Some(delay) => delay,
        };

        delay = delay.min(MAX_BREAKOUT_STOP_DELAY);

        let stops_at = Timestamp::now() + TimeDelta::seconds(delay.as_secs() as i64);

        self.message_router
            .conference
            .serialize_and_broadcast(
                BREAKOUT_MODULE_ID,
                origin.transaction_id,
                BreakoutEvent::CloseNotice {
                    issued_by: origin.id,
                    stops_at,
                },
            )
            .await?;

        self.breakout_config
            .as_mut()
            .context("breakout rooms should be active")?
            .set_expiry(delay);

        Ok(())
    }

    /// Switch between the main and/or breakout rooms
    async fn switch_room(
        &mut self,
        participant_id: ParticipantId,
        new_room: RoomKind,
        origin: EventOrigin,
    ) -> Result<(), SignalingModuleError<BreakoutError>> {
        let Some(breakout_config) = &self.breakout_config else {
            return Err(BreakoutError::BreakoutInactive.into());
        };

        if let RoomKind::Breakout(id) = new_room
            && breakout_config
                .config
                .rooms
                .get(u64::from(id) as usize)
                .is_none()
        {
            return Err(BreakoutError::UnknownBreakoutId.into());
        }

        self.move_participant(origin, participant_id, new_room)
            .await
    }

    /// Move the given participant to the breakout room
    ///
    /// Providing no breakout room will move the participant to the main room.
    ///
    /// Sends a [`BreakoutEvent::SwitchedRoom`] to the moved participant and [`BreakoutEvent::ParticipantSwitchedRoom`] to
    /// all participants.
    #[tracing::instrument(level = "debug", skip(self))]
    async fn move_participant(
        &mut self,
        origin: EventOrigin,
        participant_id: ParticipantId,
        room: RoomKind,
    ) -> Result<(), SignalingModuleError<BreakoutError>> {
        let Some(participant_state) = self.participants.all_unfiltered.get_mut(&participant_id)
        else {
            return Err(anyhow!("Received message from non-existent participant").into());
        };

        let previous_room = participant_state.room;

        if previous_room == room {
            return Err(BreakoutError::AlreadyInRoom.into());
        }

        participant_state.room = room;
        let connections: Vec<ConnectionId> = participant_state.connections().collect();

        let mut own_data = BTreeMap::new();
        let mut peer_events = BTreeMap::new();
        let mut peer_data = BTreeMap::new();

        let actions = self.broadcast_event_to_modules(
            origin,
            room,
            DynBroadcastEvent::SwitchRoom {
                participant_id,
                old_room: previous_room,
                new_room: room,
                own_data: &mut own_data,
                peer_data: &mut peer_data,
                peer_events: &mut peer_events,
            },
        );

        // send switched room event to all our connections
        for &connection_id in &connections {
            let own_data = own_data.remove(&connection_id).unwrap_or_default();
            self.message_router
                .conference
                .serialize_and_send(
                    [connection_id],
                    BREAKOUT_MODULE_ID,
                    origin.transaction_id(),
                    BreakoutEvent::SwitchedRoom {
                        own_data,
                        old_room: previous_room,
                        new_room: room,
                        peer_data: peer_data.clone(),
                    },
                )
                .await?;
        }

        actions.handle_requested_messages(self).await;

        for (&other_id, state) in self
            .participants
            .connected()
            .iter()
            .filter(|(p, _)| *p != &participant_id)
        {
            let peer_event = peer_events.remove(&other_id);

            self.message_router
                .conference
                .serialize_and_send(
                    state.connections(),
                    BREAKOUT_MODULE_ID,
                    None,
                    BreakoutEvent::ParticipantSwitchedRoom {
                        participant_id,
                        old_room: previous_room,
                        new_room: room,
                        module_data: peer_event.unwrap_or_default(),
                    },
                )
                .await?;
        }
        Ok(())
    }

    pub(crate) async fn breakout_expired(&mut self) {
        if let Err(err) = self.close_breakout_rooms(EventOrigin::Internal).await {
            tracing::error!("Fatal error on breakout expiry: {err:?}");
        }
    }

    /// Add information about peers
    pub(crate) fn breakout_peer_data(&self, state: &ParticipantState) -> BreakoutPeerModuleData {
        BreakoutPeerModuleData { room: state.room }
    }

    /// Close all breakout rooms and move the participants back to the main room
    pub(crate) async fn close_breakout_rooms(
        &mut self,
        origin: EventOrigin,
    ) -> Result<(), SignalingModuleError<BreakoutError>> {
        self.message_router
            .conference
            .serialize_and_broadcast(
                BREAKOUT_MODULE_ID,
                origin.transaction_id(),
                BreakoutEvent::Closing {
                    issued_by: origin.participant_id(),
                },
            )
            .await?;

        self.broadcast_event_to_modules(origin, RoomKind::Main, DynBroadcastEvent::BreakoutClosing)
            .handle_requested_messages(self)
            .await;

        let all_participants = self
            .participants
            .all_unfiltered
            .keys()
            .copied()
            .collect::<Vec<_>>();

        // move all participants back to the main room, ignore non-fatal errors
        let span = tracing::debug_span!("move_participants");
        for participant_id in all_participants {
            if let Err(SignalingModuleError::Fatal(error)) = self
                .move_participant(origin, participant_id, RoomKind::Main)
                .instrument(span.clone())
                .await
            {
                return Err(SignalingModuleError::Fatal(error));
            }
        }

        self.breakout_config = None;

        self.message_router
            .conference
            .serialize_and_broadcast(
                BREAKOUT_MODULE_ID,
                origin.transaction_id(),
                BreakoutEvent::Closed,
            )
            .await?;

        self.broadcast_event_to_modules(origin, RoomKind::Main, DynBroadcastEvent::BreakoutClosed)
            .handle_requested_messages(self)
            .await;

        Ok(())
    }

    /// Returns when the breakout rooms have expired
    pub(crate) async fn check_breakout_timeout(breakout_state: &mut Option<BreakoutState>) {
        match breakout_state {
            Some(state) => state.wait_for_expiry().await,
            None => std::future::pending().await,
        }
    }

    /// Attach the breakout join info data to the given [`ModuleData`]
    pub(crate) fn add_breakout_module_data(
        &self,
        module_data: &mut ModuleData,
        current_room: RoomKind,
    ) {
        let Some(breakout_config) = &self.breakout_config else {
            return;
        };

        let mut rooms = Vec::new();

        for (id, room) in breakout_config.config.rooms.iter().enumerate() {
            rooms.push(BreakoutRoom {
                id: BreakoutId::from(id as u64),
                name: room.name.clone(),
            })
        }

        if let Err(e) = module_data.insert(&BreakoutModuleData {
            room: current_room,
            rooms,
            expires: breakout_config.expires_at,
        }) {
            tracing::error!("Failed to add breakout module data to join success: {e:?}")
        }
    }
}
