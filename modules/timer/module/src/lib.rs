// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{BTreeSet, HashMap},
    time::Duration,
};

use anyhow::Context;
use opentalk_roomserver_signaling::{
    module_context::{ChannelDroppedError, ModuleContext},
    signaling_module::{
        JoinInfo, NoOp, PeerJoinInfoMap, SignalingModule, SignalingModuleInitData, SwitchInfo,
    },
};
use opentalk_roomserver_types::{
    breakout::BreakoutRoom, connection_id::ConnectionId, room_kind::RoomKind,
    signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_timer::{
    Kind, Start, StopKind, TIMER_MODULE_ID, TimerCommand, TimerConfig, TimerError, command,
    event::{Started, Stopped, TimerEvent, updated_ready_status::UpdatedReadyStatus},
    peer_state::TimerPeerState,
    state::TimerState,
};
use opentalk_types_common::{modules::ModuleId, time::Timestamp};
use opentalk_types_signaling::ParticipantId;

use crate::timer::Timer;

mod timer;

#[derive(Debug)]
pub struct TimerModule {
    timers: HashMap<RoomKind, Option<Timer>>,
    ready_participants: HashMap<RoomKind, BTreeSet<ParticipantId>>,
}

impl SignalingModule for TimerModule {
    const NAMESPACE: ModuleId = TIMER_MODULE_ID;

    type Incoming = TimerCommand;

    type Outgoing = TimerEvent;

    type Internal = NoOp;

    type Loopback = Result<Stopped, ChannelDroppedError>;

    type JoinInfo = TimerState;

    type PeerJoinInfo = TimerPeerState;

    type Error = TimerError;

    fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self {
            timers: HashMap::from([(RoomKind::Main, None)]),
            ready_participants: HashMap::from([(RoomKind::Main, BTreeSet::new())]),
        })
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<JoinInfo<Self>, SignalingModuleError<Self::Error>> {
        let timer = self
            .timers
            .get(&ctx.room)
            .with_context(|| format!("Room '{:?}' does not exist in timers", ctx.room))?;

        // Do not add JoinSuccess or PeerJoinInfo when there is no running timer
        let Some(timer) = timer else {
            return Ok(JoinInfo {
                join_success: None,
                peer: PeerJoinInfoMap::default(),
                participant_states: PeerJoinInfoMap::default(),
            });
        };

        if timer.config.ready_check_enabled {
            // Joining participants might already be ready when reconnecting
            let ready_status = self
                .ready_participants
                .get(&ctx.room)
                .with_context(|| {
                    format!(
                        "Room '{:?}' does not exist in participant ready state",
                        ctx.room
                    )
                })?
                .contains(&participant_id);

            // Append ready state when the running timer has ready check enabled
            let mut peer = PeerJoinInfoMap::default();
            peer.insert_for_all(ctx, TimerPeerState { ready_status })?;
            Ok(JoinInfo {
                join_success: Some(TimerState {
                    config: timer.config.clone(),
                    ready_status: Some(ready_status),
                }),
                participant_states: PeerJoinInfoMap::default(),
                peer,
            })
        } else {
            Ok(JoinInfo {
                join_success: Some(TimerState {
                    config: timer.config.clone(),
                    ready_status: None,
                }),
                participant_states: PeerJoinInfoMap::default(),
                peer: PeerJoinInfoMap::default(),
            })
        }
    }

    #[allow(unused_variables)]
    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    fn on_breakout_start(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        rooms: &[BreakoutRoom],
        _duration: Option<Duration>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        for room in rooms {
            let room = RoomKind::Breakout(room.id);
            self.timers.insert(room, None);
            self.ready_participants.insert(room, BTreeSet::new());
        }
        Ok(())
    }

    fn on_breakout_switch(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _old_room: RoomKind,
        new_room: RoomKind,
    ) -> Result<SwitchInfo<Self>, SignalingModuleError<Self::Error>> {
        let timer = self
            .timers
            .get(&new_room)
            .with_context(|| format!("Room '{new_room:?}' does not exist in timers"))?;

        // Timer is disabled, send empty JoinInfo
        let Some(timer) = timer else {
            return Ok(SwitchInfo::<Self>::new());
        };

        let ready_status = if timer.config.ready_check_enabled {
            // Joining participants might already be ready when they were in the
            // room before
            Some(
                self.ready_participants
                    .get(&new_room)
                    .with_context(|| {
                        format!("Room '{new_room:?}' does not exist in participant ready state")
                    })?
                    .contains(&participant_id),
            )
        } else {
            None
        };
        let timer_state = Some(TimerState {
            config: timer.config.clone(),
            ready_status,
        });

        let connections = ctx
            .participant_state(participant_id)
            .with_context(|| format!("Participant '{participant_id}' does not have state"))?
            .connections();
        let switch_info = connections.map(|con| (con, timer_state.clone())).collect();

        Ok(switch_info)
    }

    fn on_breakout_closed(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        self.ready_participants
            .retain(|room, _| *room == RoomKind::Main);
        self.timers.retain(|room, _| *room == RoomKind::Main);
        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        _connection_id: ConnectionId,
        payload: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match payload {
            TimerCommand::Start(start) => self.start_timer(ctx, sender, start)?,
            TimerCommand::Stop { reason } => self.stop_timer(ctx, sender, reason)?,
            TimerCommand::UpdateReadyStatus { status } => {
                self.update_ready_status(ctx, sender, status)?;
            }
        }
        Ok(())
    }

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        self.remove_timer(ctx.room)?;

        match event {
            Ok(stopped) => {
                ctx.send_ws_message(
                    ctx.participants.filter().room(ctx.room).ids(),
                    TimerEvent::Stopped(stopped),
                )?;
            }
            Err(..) => {
                ctx.send_ws_message(
                    ctx.participants.filter().room(ctx.room).ids(),
                    TimerEvent::Error(TimerError::Internal),
                )?;
            }
        }
        Ok(())
    }
}

impl TimerModule {
    fn start_timer(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        start: Start,
    ) -> Result<(), SignalingModuleError<<TimerModule as SignalingModule>::Error>> {
        if !ctx.is_moderator(sender) {
            return Err(TimerError::InsufficientPermissions.into());
        }

        let timer = self
            .timers
            .get_mut(&ctx.room)
            .with_context(|| format!("Room '{:?}' does not exist in timers", ctx.room))?;
        if timer.is_some() {
            return Err(TimerError::TimerAlreadyRunning.into());
        }

        let started_at = ctx.timestamp;
        let mut tx_cancel = None;
        let kind = match start.kind {
            command::Kind::Stopwatch => Kind::Stopwatch,
            command::Kind::Countdown { duration } => {
                let signed_duration = duration
                    .try_into()
                    .map_err(|_| TimerError::InvalidDuration)?;
                let ends_at = started_at
                    .checked_add_signed(chrono::Duration::seconds(signed_duration))
                    .ok_or(TimerError::InvalidDuration)?;

                let tx = ctx.loopback_after(Duration::from_secs(duration), || Stopped {
                    kind: StopKind::Expired,
                    reason: None,
                });
                tx_cancel = Some(tx);

                Kind::Countdown {
                    ends_at: Timestamp::from(ends_at),
                }
            }
        };

        *timer = Some(Timer {
            config: TimerConfig {
                started_at,
                kind,
                style: start.style.clone(),
                title: start.title.clone(),
                ready_check_enabled: start.enable_ready_check,
            },
            tx_cancel,
        });

        ctx.send_ws_message(
            ctx.participants.filter().room(ctx.room).ids(),
            TimerEvent::Started(Started {
                config: TimerConfig {
                    started_at,
                    kind,
                    style: start.style,
                    title: start.title,
                    ready_check_enabled: start.enable_ready_check,
                },
            }),
        )?;
        Ok(())
    }

    fn stop_timer(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        reason: Option<String>,
    ) -> Result<(), SignalingModuleError<<TimerModule as SignalingModule>::Error>> {
        if !ctx.is_moderator(sender) {
            return Err(TimerError::InsufficientPermissions.into());
        }

        if let Some(mut timer) = self.remove_timer(ctx.room)? {
            let stopped = Stopped {
                kind: StopKind::ByModerator(sender),
                reason,
            };
            if let Some(tx) = timer.tx_cancel.take() {
                if tx.send(stopped).is_err() {
                    tracing::debug!("Timer cancel sender has been dropped");
                }
            } else {
                // If there is no cancel sender, this means the timer does not use a
                // loopback task (e.g. stopwatch). In this case we can simply notify
                // the participants that the timer was cancelled.
                ctx.send_ws_message(
                    ctx.participants.filter().room(ctx.room).ids(),
                    TimerEvent::Stopped(stopped),
                )?;
            }
        }

        Ok(())
    }

    /// Removes the timer and the associated ready state
    fn remove_timer(
        &mut self,
        room: RoomKind,
    ) -> Result<Option<Timer>, SignalingModuleError<<TimerModule as SignalingModule>::Error>> {
        let timer = self
            .timers
            .get_mut(&room)
            .with_context(|| format!("Room '{room:?}' does not exist in timers"))?;
        let ready_states = self
            .ready_participants
            .get_mut(&room)
            .with_context(|| format!("Room '{room:?}' does not exist in ready states",))?;
        if let Some(timer) = timer.take() {
            ready_states.clear();
            return Ok(Some(timer));
        }
        Ok(None)
    }

    fn update_ready_status(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        ready: bool,
    ) -> Result<(), SignalingModuleError<<TimerModule as SignalingModule>::Error>> {
        let timer = self
            .timers
            .get_mut(&ctx.room)
            .with_context(|| format!("Room '{:?}' does not exist in timers", ctx.room))?;

        let Some(timer) = timer else {
            return Err(TimerError::TimerNotRunning)?;
        };

        if !timer.config.ready_check_enabled {
            return Err(TimerError::ReadyCheckNotEnabled)?;
        }

        let ready_participants = self
            .ready_participants
            .get_mut(&ctx.room)
            .with_context(|| {
                format!(
                    "Room '{:?}' does not exist in participant ready state",
                    ctx.room
                )
            })?;

        let changed = if ready {
            ready_participants.insert(sender)
        } else {
            ready_participants.remove(&sender)
        };
        if changed {
            ctx.send_ws_message(
                ctx.participants.filter().room(ctx.room).ids(),
                TimerEvent::UpdatedReadyStatus(UpdatedReadyStatus {
                    participant_id: sender,
                    status: ready,
                }),
            )?;
        }

        Ok(())
    }
}
