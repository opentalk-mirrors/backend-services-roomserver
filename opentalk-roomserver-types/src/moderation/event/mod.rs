// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

pub use crate::moderation::event::error::ModerationError;

mod error;

/// Events sent out by the `moderation` module
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum ModerationEvent {
    /// Sent to a participant when they are accepted by the moderator from the waiting room
    Accepted,

    /// An error happened when executing a `moderation` command
    Error(ModerationError),
}

impl From<ModerationError> for ModerationEvent {
    fn from(value: ModerationError) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn accepted() {
        let expected = json!({"message": "accepted"});

        let produced = serde_json::to_value(ModerationEvent::Accepted).unwrap();

        assert_eq!(expected, produced);
    }
}
