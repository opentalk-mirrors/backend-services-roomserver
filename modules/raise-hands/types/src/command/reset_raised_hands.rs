// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// Reset raised hands for the meeting
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResetRaisedHands {
    /// An optional single participant to reset the raised hand for
    #[serde(
        default,
        with = "opentalk_types_common::collections::one_or_many_btree_set_option"
    )]
    pub target: Option<BTreeSet<ParticipantId>>,
}
