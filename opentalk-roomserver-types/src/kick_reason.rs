// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KickReason {
    Kicked,
    Debriefed,
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_kicked() {
        let reason = KickReason::Kicked;
        let produced = serde_json::to_string_pretty(&reason).unwrap();

        assert_snapshot!(produced, @r#""kicked""#);
    }

    #[test]
    fn deserialize_kicked() {
        let produced: KickReason = serde_json::from_value(json!("kicked")).unwrap();
        let expected = KickReason::Kicked;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_debrief() {
        let reason = KickReason::Debriefed;
        let produced = serde_json::to_string_pretty(&reason).unwrap();

        assert_snapshot!(produced, @r#""debriefed""#);
    }

    #[test]
    fn deserialize_debrief() {
        let produced: KickReason = serde_json::from_value(json!("debriefed")).unwrap();
        let expected = KickReason::Debriefed;

        assert_eq!(produced, expected);
    }
}
