// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use serde::{Deserialize, Serialize};

use crate::{PollId, command::Vote, event::PollsEvent};

/// Commands received by the `polls` module
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum PollsCommand {
    /// Start a poll
    Start {
        /// The description of the poll topic
        topic: String,

        /// True if the poll is live
        #[serde(default)]
        live: bool,

        /// True if the poll accepts multiple choices
        #[serde(default)]
        multiple_choice: bool,

        /// The choices of the poll
        choices: Vec<String>,

        /// The duration of the poll
        #[serde(with = "opentalk_types_common::utils::duration_seconds")]
        duration: Duration,
    },

    /// Cast a vote
    Vote(Vote),

    /// Finish the poll
    Finish {
        /// The id of the poll
        id: PollId,
    },
}

impl CreateReplica<PollsEvent> for PollsCommand {
    fn replicate(&self) -> Option<PollsEvent> {
        match self {
            PollsCommand::Vote(vote) => Some(PollsEvent::Voted(vote.clone())),
            _ => None,
        }
    }
}

impl From<Vote> for PollsCommand {
    fn from(value: Vote) -> Self {
        Self::Vote(value)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, time::Duration};

    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{ChoiceId, PollId, command::Choices};

    #[test]
    fn start() {
        let json = json!({
            "action": "start",
            "topic": "abc",
            "live": true,
            "choices": ["a", "b", "c"],
            "duration": 30
        });

        let message: PollsCommand = serde_json::from_value(json).unwrap();

        if let PollsCommand::Start {
            topic,
            live,
            multiple_choice,
            choices,
            duration,
        } = message
        {
            assert_eq!(topic, "abc");
            assert!(live);
            assert!(!multiple_choice);
            assert_eq!(choices, vec!["a", "b", "c"]);
            assert_eq!(duration, Duration::from_secs(30));
        } else {
            panic!("expected PollsCommand::Start but got: {message:?}");
        }
    }

    #[test]
    fn single_choice_vote() {
        let json = json!({
           "action": "vote",
           "poll_id": "00000000-0000-0000-0000-000000000000",
           "choice_id": 321,
        });

        let message: PollsCommand = serde_json::from_value(json).unwrap();

        if let PollsCommand::Vote(Vote { poll_id, choices }) = message {
            assert_eq!(poll_id, PollId::nil());
            assert_eq!(
                choices,
                Choices::Single {
                    choice_id: ChoiceId::from(321),
                }
            );
        } else {
            panic!("Expected PollsCommand::Vote, got: {message:?}");
        }
    }

    #[test]
    fn multiple_choice_vote() {
        let json = json!({
           "action": "vote",
           "poll_id": "00000000-0000-0000-0000-000000000000",
           "choice_ids": [322, 322, 323]
        });

        let message: PollsCommand = serde_json::from_value(json).unwrap();

        if let PollsCommand::Vote(Vote { poll_id, choices }) = message {
            assert_eq!(poll_id, PollId::nil());
            assert_eq!(
                choices,
                Choices::Multiple {
                    choice_ids: BTreeSet::from([ChoiceId::from(322), ChoiceId::from(323)]),
                }
            );
        } else {
            panic!("Expected PollsCommand::Vote, got: {message:?}");
        }
    }

    #[test]
    fn conflicting_choice_vote() {
        let json = json!({
           "action": "vote",
           "poll_id": "00000000-0000-0000-0000-000000000000",
           "choice_id": 321,
           "choice_ids": [322, 322, 323]
        });

        let message: PollsCommand = serde_json::from_value(json).unwrap();

        if let PollsCommand::Vote(Vote { poll_id, choices }) = message {
            assert_eq!(poll_id, PollId::nil());
            assert_eq!(
                choices,
                Choices::Single {
                    choice_id: ChoiceId::from(321),
                }
            );
        } else {
            panic!("Expected PollsCommand::Vote, got: {message:?}");
        }
    }

    #[test]
    fn abstain() {
        let json = json!({
           "action": "vote",
           "poll_id": "00000000-0000-0000-0000-000000000000"
        });

        let message: PollsCommand = serde_json::from_value(json).unwrap();

        if let PollsCommand::Vote(Vote { poll_id, choices }) = message {
            assert_eq!(poll_id, PollId::nil());
            assert_eq!(
                choices,
                Choices::Multiple {
                    choice_ids: BTreeSet::new(),
                }
            );
        } else {
            panic!("Expected PollsCommand::Vote, got: {message:?}");
        }
    }

    #[test]
    fn finish() {
        let json = json!({
            "action": "finish",
            "id": "00000000-0000-0000-0000-000000000000",
        });

        let message: PollsCommand = serde_json::from_value(json).unwrap();

        assert_eq!(message, PollsCommand::Finish { id: PollId::nil() });
    }
}
