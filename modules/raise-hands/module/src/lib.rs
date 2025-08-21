// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{BTreeSet, HashMap};

use anyhow::Context as _;
use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{JoinInfo, NoOp, SignalingModule, SignalingModuleInitData, SwitchInfo},
};
use opentalk_roomserver_types::{
    breakout::BreakoutRoom, connection_id::ConnectionId, room_kind::RoomKind,
    signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_raise_hands::{
    RAISE_HANDS_MODULE_ID,
    command::{RaiseHandsCommand, ResetRaisedHands},
    event::{RaiseHandsError, RaiseHandsEvent},
    state::{RaiseHandsState, RaisedHandState},
};
use opentalk_types_common::{modules::ModuleId, time::Timestamp};
use opentalk_types_signaling::ParticipantId;

pub struct RaiseHandsModule {
    /// The raised hands state for each room. If a room is not present in this map,
    /// raised hands are not enabled for that room.
    raised_hands: HashMap<RoomKind, HashMap<ParticipantId, Timestamp>>,
}

impl SignalingModule for RaiseHandsModule {
    const NAMESPACE: ModuleId = RAISE_HANDS_MODULE_ID;

    type Incoming = RaiseHandsCommand;

    type Outgoing = RaiseHandsEvent;

    type Internal = NoOp;

    type Loopback = ();

    type JoinInfo = RaiseHandsState;

    type PeerJoinInfo = ();

    type Error = RaiseHandsError;

    fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self {
            raised_hands: HashMap::from_iter([(RoomKind::Main, HashMap::new())]),
        })
    }

    #[allow(unused_variables)]
    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        is_first_connection: bool,
    ) -> Result<JoinInfo<Self>, SignalingModuleError<Self::Error>> {
        let raised_hands = self.raised_hands_state(ctx.room);
        let join_info = JoinInfo {
            join_success: Some(RaiseHandsState {
                raise_hands_enabled: raised_hands.is_some(),
                raised_hands,
            }),
            ..Default::default()
        };
        Ok(join_info)
    }

    #[allow(unused_variables)]
    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        if let Some(raised_hands) = self.raised_hands.get_mut(&ctx.room) {
            raised_hands.remove(&participant_id);
        };

        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        _connection_id: ConnectionId,
        content: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match content {
            RaiseHandsCommand::EnableRaiseHands => self.enable_raise_hands(ctx, sender),
            RaiseHandsCommand::DisableRaiseHands => self.disable_raise_hands(ctx, sender),
            RaiseHandsCommand::RaiseHand => self.raise_hand(ctx, sender),
            RaiseHandsCommand::LowerHand => self.lower_hand(ctx, sender),
            RaiseHandsCommand::ResetRaisedHands(ResetRaisedHands { target }) => {
                self.reset_raised_hands(ctx, sender, target)
            }
        }
    }

    fn on_breakout_start(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        rooms: &[BreakoutRoom],
        _duration: Option<std::time::Duration>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        // Enable raised hands for breakout rooms when enabled in the main room.
        if self.raised_hands.contains_key(&RoomKind::Main) {
            self.raised_hands.extend(
                rooms
                    .iter()
                    .map(|room| (RoomKind::Breakout(room.id), HashMap::new())),
            );
        }
        Ok(())
    }

    fn on_breakout_switch(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        old_room: RoomKind,
        _new_room: RoomKind,
    ) -> Result<SwitchInfo<Self>, SignalingModuleError<Self::Error>> {
        // Reset raised hand of the participant when switching rooms
        self.raised_hands
            .entry(old_room)
            .and_modify(|raised_hands| {
                raised_hands.remove(&participant_id);
            });
        let raised_hands = self.raised_hands_state(ctx.room);
        let moderation_state = RaiseHandsState {
            raise_hands_enabled: raised_hands.is_some(),
            raised_hands,
        };
        let switch_success = ctx
            .participant_state(participant_id)
            .with_context(|| format!("Missing state for participant '{participant_id}'^"))?
            .connections()
            .map(|con| (con, Some(moderation_state.clone())))
            .collect();
        Ok(SwitchInfo {
            switch_success,
            ..Default::default()
        })
    }

    fn on_breakout_closed(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        self.raised_hands.retain(|room, _| *room == RoomKind::Main);
        Ok(())
    }
}

impl RaiseHandsModule {
    fn enable_raise_hands(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<RaiseHandsError>> {
        if !ctx.is_moderator(sender) {
            return Err(RaiseHandsError::InsufficientPermissions.into());
        }

        self.raised_hands.entry(ctx.room).or_default();

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            RaiseHandsEvent::RaiseHandsEnabled { issued_by: sender },
        )?;

        Ok(())
    }

    fn disable_raise_hands(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<RaiseHandsError>> {
        if !ctx.is_moderator(sender) {
            return Err(RaiseHandsError::InsufficientPermissions.into());
        }

        self.raised_hands.remove(&ctx.room);

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            RaiseHandsEvent::RaiseHandsDisabled { issued_by: sender },
        )?;

        Ok(())
    }

    fn raise_hand(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
    ) -> Result<(), SignalingModuleError<RaiseHandsError>> {
        let raised_hands = self
            .raised_hands
            .get_mut(&ctx.room)
            .ok_or(RaiseHandsError::RaiseHandsDisabled)?;

        raised_hands
            .entry(participant_id)
            .and_modify(|timestamp| *timestamp = ctx.timestamp)
            .or_insert(ctx.timestamp);

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            RaiseHandsEvent::HandRaised {
                participant: participant_id,
            },
        )?;

        Ok(())
    }

    fn lower_hand(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
    ) -> Result<(), SignalingModuleError<RaiseHandsError>> {
        let raised_hands = self
            .raised_hands
            .get_mut(&ctx.room)
            .ok_or(RaiseHandsError::RaiseHandsDisabled)?;

        raised_hands.remove(&participant_id);

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            RaiseHandsEvent::HandLowered {
                participant: participant_id,
            },
        )?;

        Ok(())
    }

    fn reset_raised_hands(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        targets: Option<BTreeSet<ParticipantId>>,
    ) -> Result<(), SignalingModuleError<RaiseHandsError>> {
        if !ctx.is_moderator(sender) {
            return Err(RaiseHandsError::InsufficientPermissions.into());
        }

        let raised_hands = self
            .raised_hands
            .get_mut(&ctx.room)
            .ok_or(RaiseHandsError::RaiseHandsDisabled)?;

        let mut participants: BTreeSet<ParticipantId>;
        if let Some(targets) = targets {
            participants = BTreeSet::new();
            for participant_id in targets {
                if raised_hands.remove(&participant_id).is_some() {
                    participants.insert(participant_id);
                }
            }
        } else {
            participants = raised_hands.keys().cloned().collect();
            raised_hands.clear();
        }

        if !participants.is_empty() {
            ctx.send_ws_message(
                ctx.participants.in_room(ctx.room).connected().ids(),
                RaiseHandsEvent::RaisedHandResetByModerator {
                    issued_by: sender,
                    participants,
                },
            )?;
        }

        Ok(())
    }

    fn raised_hands_state(&self, room: RoomKind) -> Option<BTreeSet<RaisedHandState>> {
        self.raised_hands.get(&room).map(|raised_hands| {
            raised_hands
                .iter()
                .map(|(&participant_id, &raised_at)| RaisedHandState {
                    participant_id,
                    raised_at,
                })
                .collect()
        })
    }
}
