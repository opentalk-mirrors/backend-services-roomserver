// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::users::GroupName;
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_chat::Scope;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ChatId {
    /// Global scope for chat
    Global,

    /// Group scope for chat
    Group(GroupName),

    /// Private scope for chat
    Private(PrivateChatId),
}

impl ChatId {
    pub fn from_scope_and_source(scope: Scope, source: ParticipantId) -> Self {
        match scope {
            Scope::Global => Self::Global,
            Scope::Group(group_name) => Self::Group(group_name),
            Scope::Private(participant_id) => {
                Self::Private(PrivateChatId::new(source, participant_id))
            }
        }
    }

    /// Returns `true` if the chat id is [`Private`].
    ///
    /// [`Private`]: ChatId::Private
    #[must_use]
    pub fn is_private(&self) -> bool {
        matches!(self, Self::Private(..))
    }

    pub fn as_private(&self) -> Option<PrivateChatId> {
        if let Self::Private(v) = self {
            Some(*v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PrivateChatId(ParticipantId, ParticipantId);

impl PrivateChatId {
    pub fn new(participant_a: ParticipantId, participant_b: ParticipantId) -> Self {
        if participant_a < participant_b {
            Self(participant_a, participant_b)
        } else {
            Self(participant_b, participant_a)
        }
    }

    /// Returns the other participant in the chat.
    ///
    /// If the provided `participant` matches any member of this chat, the method returns the other participant.
    pub fn other(&self, participant: ParticipantId) -> ParticipantId {
        if self.0 == participant {
            self.1
        } else {
            self.0
        }
    }

    /// Returns a scope for this chat for the given participant.
    ///
    /// Since [`Scope::Private`] contains one [`ParticipantId`] and the other participant is implicit,
    /// we need to create the Scope for this implicit participant.
    pub fn to_scope(&self, base: ParticipantId) -> Scope {
        Scope::Private(self.other(base))
    }

    pub fn participants(&self) -> [ParticipantId; 2] {
        [self.0, self.1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_id_global() {
        let scope = Scope::Global;
        let source = ParticipantId::from_u128(0x1);
        let chat_id = ChatId::from_scope_and_source(scope, source);

        assert_eq!(chat_id, ChatId::Global);
    }

    #[test]
    fn chat_id_group() {
        let group_name = GroupName::from("group_name".to_string());
        let scope = Scope::Group(group_name.clone());
        let source = ParticipantId::from_u128(0x1);
        let chat_id = ChatId::from_scope_and_source(scope, source);

        assert_eq!(chat_id, ChatId::Group(group_name));
    }

    #[test]
    fn chat_id_private() {
        let source = ParticipantId::from_u128(0x1);
        let participant = ParticipantId::from_u128(0x2);
        let scope = Scope::Private(participant);
        let chat_id = ChatId::from_scope_and_source(scope, source);

        let expected_set = PrivateChatId::new(source, participant);

        assert_eq!(chat_id, ChatId::Private(expected_set));
    }

    #[test]
    fn chat_id_private_order_independence() {
        let participant_a = ParticipantId::from_u128(0x2);
        let participant_b = ParticipantId::from_u128(0x1);

        let chat_id_ab = ChatId::Private(PrivateChatId::new(participant_a, participant_b));
        let chat_id_ba = ChatId::Private(PrivateChatId::new(participant_b, participant_a));

        assert_eq!(chat_id_ab, chat_id_ba);
    }
}
