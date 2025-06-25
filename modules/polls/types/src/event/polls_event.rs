// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::{
    Results,
    command::Vote,
    event::{Error, Started},
};

/// Events sent out by the `polls` module
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "message")]
pub enum PollsEvent {
    /// The poll has started
    Started(Started),

    /// Live update of the poll results
    LiveUpdate(Results),

    /// A vote was cast on a different device
    Voted(Vote),

    /// The poll is completed
    Done(Results),

    /// An error happened when executing a `polls` command
    Error(Error),
}

impl From<Started> for PollsEvent {
    fn from(value: Started) -> Self {
        Self::Started(value)
    }
}

impl From<Error> for PollsEvent {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod serde_tests {
    use std::{collections::BTreeSet, time::Duration};

    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{Choice, ChoiceId, Item, PollId, command::Choices};

    #[test]
    fn started() {
        let started = PollsEvent::Started(Started {
            id: PollId::nil(),
            topic: "polling".into(),
            live: true,
            multiple_choice: false,
            choices: vec![
                Choice {
                    id: ChoiceId::from(0),
                    content: "yes".into(),
                },
                Choice {
                    id: ChoiceId::from(1),
                    content: "no".into(),
                },
            ],
            duration: Duration::from_millis(10000),
        });

        assert_eq!(
            serde_json::to_value(started).unwrap(),
            json!({
                "message": "started",
                "id": "00000000-0000-0000-0000-000000000000",
                "topic": "polling",
                "live": true,
                "multiple_choice": false,
                "choices": [
                    {
                        "id": 0,
                        "content": "yes"
                    },
                    {
                        "id": 1,
                        "content": "no"
                    }
                ],
                "duration": 10
            })
        );
    }

    #[test]
    fn live_update() {
        let live_update = PollsEvent::LiveUpdate(Results {
            id: PollId::nil(),
            results: vec![
                Item {
                    id: ChoiceId::from(0),
                    count: 32,
                },
                Item {
                    id: ChoiceId::from(1),
                    count: 64,
                },
            ],
        });

        assert_eq!(
            serde_json::to_value(live_update).unwrap(),
            json!({
                "message": "live_update",
                "id": "00000000-0000-0000-0000-000000000000",
                "results": [
                    {
                        "id": 0,
                        "count": 32
                    },
                    {
                        "id": 1,
                        "count": 64
                    }
                ]
            })
        );
    }

    #[test]
    fn voted() {
        let voted = PollsEvent::Voted(Vote {
            poll_id: PollId::nil(),
            choices: Choices::Multiple {
                choice_ids: BTreeSet::from_iter([ChoiceId::from_u32(0), ChoiceId::from_u32(1)]),
            },
        });

        assert_eq!(
            serde_json::to_value(voted).unwrap(),
            json!({
                "message": "voted",
                "poll_id": "00000000-0000-0000-0000-000000000000",
                "choice_ids": [0, 1],
            })
        )
    }

    #[test]
    fn done() {
        let done = PollsEvent::Done(Results {
            id: PollId::nil(),
            results: vec![
                Item {
                    id: ChoiceId::from(0),
                    count: 32,
                },
                Item {
                    id: ChoiceId::from(1),
                    count: 64,
                },
            ],
        });

        assert_eq!(
            serde_json::to_value(done).unwrap(),
            json!({
                "message": "done",
                "id": "00000000-0000-0000-0000-000000000000",
                "results": [
                    {
                        "id": 0,
                        "count": 32
                    },
                    {
                        "id": 1,
                        "count": 64
                    }
                ]
            })
        );
    }
}
