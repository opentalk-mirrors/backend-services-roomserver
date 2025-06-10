// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{BTreeMap, HashMap};

use chat_id::{ChatId, PrivateChatId};
use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{JoinInfo, SignalingModule, SignalingModuleInitData},
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId, signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_chat::{
    CHAT_MODULE_ID,
    command::ChatCommand,
    event::{ChatError, ChatEvent},
};
use opentalk_types_common::{modules::ModuleId, time::Timestamp};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_chat::{
    MessageId, Scope,
    command::{SendMessage, SetLastSeenTimestamp},
    event::{ChatDisabled, ChatEnabled, HistoryCleared, MessageSent},
    peer_state::ChatPeerState,
    state::{ChatState, PrivateHistory, StoredMessage},
};

pub mod chat_id;

#[derive(Debug)]
pub struct ChatModule {
    enabled: bool,

    history: HashMap<ChatId, Vec<StoredMessage>>,

    /// Records for each participant in which chat they are participating and up
    /// until which time they read messages.
    chat_state: HashMap<ParticipantId, HashMap<ChatId, Option<Timestamp>>>,
}

impl SignalingModule for ChatModule {
    const NAMESPACE: ModuleId = CHAT_MODULE_ID;

    type Incoming = ChatCommand;

    type Outgoing = ChatEvent;

    type Loopback = ();

    type JoinInfo = ChatState;

    type PeerJoinInfo = ChatPeerState;

    type Error = ChatError;

    fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self {
            enabled: true,
            history: HashMap::default(),
            chat_state: HashMap::default(),
        })
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        p_joined: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<JoinInfo<Self>, SignalingModuleError<Self::Error>> {
        let mut join_info = JoinInfo {
            join_success: Some(self.chat_state_for_participant(p_joined)),
            ..Default::default()
        };

        join_info
            .peer
            .insert_for_all(ctx, ChatPeerState { groups: Vec::new() })?;

        Ok(join_info)
    }

    fn on_participant_disconnected(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        _participant_id: ParticipantId,
        _connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
        command: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match command {
            ChatCommand::EnableChat => {
                self.set_chat_state(ctx, participant_id, true)?;
            }
            ChatCommand::DisableChat => {
                self.set_chat_state(ctx, participant_id, false)?;
            }
            ChatCommand::SendMessage(SendMessage {
                scope: Scope::Group(_),
                ..
            }) => {
                log::warn!("Ignoring chat message to group");
            }
            ChatCommand::SendMessage(SendMessage { content, scope }) => {
                self.send_message(ctx, participant_id, content, scope)?;
            }
            ChatCommand::ClearHistory => {
                self.clear_messages(ctx, participant_id)?;
            }
            ChatCommand::SetLastSeenTimestamp(set_last_seen_timestamp) => {
                self.set_last_seen_timestamp(
                    ctx,
                    participant_id,
                    set_last_seen_timestamp.scope,
                    set_last_seen_timestamp.timestamp,
                )?;
            }
        }

        Ok(())
    }

    fn on_loopback_event(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        _event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }
}

impl ChatModule {
    fn chat_state_for_participant(&mut self, participant: ParticipantId) -> ChatState {
        ChatState {
            enabled: self.enabled,
            room_history: self.history.entry(ChatId::Global).or_default().clone(),
            groups_history: Vec::new(),
            private_history: self.private_chat_histories(participant),
            last_seen_timestamp_global: self.last_seen_global(participant),
            last_seen_timestamps_private: self.last_seen_timestamps_private(participant),
            last_seen_timestamps_group: Default::default(),
        }
    }

    fn last_seen_timestamps_private(
        &mut self,
        participant: ParticipantId,
    ) -> BTreeMap<ParticipantId, Timestamp> {
        let chats = self.chat_state.entry(participant).or_default();
        let mut last_seen_timestamps_private = BTreeMap::new();
        for (chat_id, timestamp) in chats {
            if let (ChatId::Private(private_id), Some(timestamp)) = (chat_id, timestamp) {
                last_seen_timestamps_private.insert(private_id.other(participant), *timestamp);
            }
        }
        last_seen_timestamps_private
    }

    fn private_chat_histories(&mut self, participant: ParticipantId) -> Vec<PrivateHistory> {
        if let Some(chat_states) = self.chat_state.get(&participant) {
            chat_states
                .keys()
                .filter_map(|id| {
                    id.as_private()
                        .and_then(|private_id| self.private_history(participant, private_id))
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn last_seen_global(&mut self, participant: ParticipantId) -> Option<Timestamp> {
        self.chat_state
            .entry(participant)
            .or_default()
            .get(&ChatId::Global)
            .copied()
            .flatten()
    }

    fn private_history(
        &self,
        participant: ParticipantId,
        chat_id: PrivateChatId,
    ) -> Option<PrivateHistory> {
        let Some(history) = self.history.get(&ChatId::Private(chat_id)).cloned() else {
            log::debug!("No private history found for chat: {chat_id:?}");
            return None;
        };
        log::debug!(
            "Private history found for chat `{chat_id:?}` with {} messages",
            history.len()
        );

        let correspondent = chat_id.other(participant);

        Some(PrivateHistory {
            correspondent,
            history,
        })
    }

    fn set_chat_state(
        &mut self,
        ctx: &mut ModuleContext<'_, ChatModule>,
        participant: ParticipantId,
        enabled: bool,
    ) -> Result<(), SignalingModuleError<<ChatModule as SignalingModule>::Error>> {
        if !ctx.is_moderator(participant) {
            return Err(ChatError::InsufficientPermissions.into());
        }

        self.enabled = enabled;
        let msg = if enabled {
            ChatEvent::ChatEnabled(ChatEnabled {
                issued_by: participant,
            })
        } else {
            ChatEvent::ChatDisabled(ChatDisabled {
                issued_by: participant,
            })
        };

        ctx.send_ws_message(ctx.participants.connected().iter().map(|(id, _)| *id), msg)?;
        Ok(())
    }

    fn send_message(
        &mut self,
        ctx: &mut ModuleContext<'_, ChatModule>,
        participant: ParticipantId,
        content: String,
        scope: Scope,
    ) -> Result<(), SignalingModuleError<<ChatModule as SignalingModule>::Error>> {
        if !self.enabled {
            return Err(ChatError::ChatDisabled.into());
        }

        let out_message = MessageSent {
            id: MessageId::generate(),
            source: participant,
            content,
            scope,
        };
        let stored_msg = StoredMessage {
            id: out_message.id,
            source: out_message.source,
            content: out_message.content.clone(),
            scope: out_message.scope.clone(),
            timestamp: Timestamp::now(),
        };
        let chat_id = ChatId::from_scope_and_source(out_message.scope.clone(), participant);

        self.history
            .entry(chat_id.clone())
            .or_default()
            .push(stored_msg);

        // ensure participation in the chat is recorded
        self.chat_state
            .entry(participant)
            .or_default()
            .entry(chat_id.clone())
            .or_insert(None);

        // if this is a private chat, we also record the chat for the other participant
        if let ChatId::Private(id) = &chat_id {
            self.chat_state
                .entry(id.other(participant))
                .or_default()
                .entry(chat_id.clone())
                .or_insert(None);
        }

        match &chat_id {
            ChatId::Global => {
                ctx.send_ws_message(
                    ctx.participants.connected().iter().map(|(id, _)| *id),
                    ChatEvent::MessageSent(out_message),
                )?;
            }
            ChatId::Group(_) => {}
            ChatId::Private(private_chat_id) => {
                // Since the Scope is relative to recipient, we need to calculate
                // individual scopes for each recipient of the message.
                for recipient in private_chat_id.participants() {
                    ctx.send_ws_message(
                        [recipient],
                        ChatEvent::MessageSent(MessageSent {
                            scope: private_chat_id.to_scope(recipient),
                            ..out_message.clone()
                        }),
                    )?;
                }
            }
        }
        Ok(())
    }

    fn clear_messages(
        &mut self,
        ctx: &mut ModuleContext<'_, ChatModule>,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError<<ChatModule as SignalingModule>::Error>> {
        if !ctx.is_moderator(participant) {
            return Err(ChatError::InsufficientPermissions.into());
        }

        self.history.remove(&ChatId::Global);

        ctx.send_ws_message(
            ctx.participants.connected().iter().map(|(id, _)| *id),
            ChatEvent::HistoryCleared(HistoryCleared {
                issued_by: participant,
            }),
        )?;
        Ok(())
    }

    fn set_last_seen_timestamp(
        &mut self,
        ctx: &mut ModuleContext<'_, ChatModule>,
        participant_id: ParticipantId,
        scope: Scope,
        timestamp: Timestamp,
    ) -> Result<(), SignalingModuleError<<ChatModule as SignalingModule>::Error>> {
        let chat_id = ChatId::from_scope_and_source(scope.clone(), participant_id);
        self.chat_state
            .entry(participant_id)
            .or_default()
            .insert(chat_id, Some(timestamp));

        ctx.send_ws_message(
            ctx.participants.connected().iter().map(|(id, _)| *id),
            ChatEvent::SetLastSeenTimestamp(SetLastSeenTimestamp { scope, timestamp }),
        )?;
        Ok(())
    }
}
