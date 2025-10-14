// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::module_settings::SignalingModuleSettings;
use opentalk_types_common::modules::ModuleId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ChatSettings {
    pub rate_limit: Option<RateLimitSettings>,
}

impl SignalingModuleSettings for ChatSettings {
    const NAMESPACE: ModuleId = crate::CHAT_MODULE_ID;
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RateLimitSettings {
    /// The tokens that are added to the bucket per second
    pub tokens_per_second: u64,
    /// The maximum amount of tokens that a token bucket can hold at a time
    pub token_bucket_size: u64,
    /// If a participant has sent this many requests in a second, they will be told to slow down
    #[serde(deserialize_with = "deserialize_slow_down_threshold")]
    pub slow_down_threshold: f32,
}

fn deserialize_slow_down_threshold<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = f32::deserialize(deserializer)?;
    if !(0.0..=1.0).contains(&value) {
        return Err(serde::de::Error::custom(
            "slow_down_threshold must be between 0.0 and 1.0 (inclusive)",
        ));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_chat_settings() {
        let settings = ChatSettings {
            rate_limit: Some(RateLimitSettings {
                tokens_per_second: 3,
                token_bucket_size: 10,
                slow_down_threshold: 0.8,
            }),
        };
        let raw = serde_json::to_string_pretty(&settings).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "rate_limit": {
            "tokens_per_second": 3,
            "token_bucket_size": 10,
            "slow_down_threshold": 0.8
          }
        }
        "#);
    }

    #[test]
    fn deserialize_chat_settings() {
        let json = json!({
            "rate_limit": {
                "tokens_per_second": 3,
                "token_bucket_size": 10,
                "slow_down_threshold": 0.8
            }
        });
        let produced: ChatSettings = serde_json::from_value(json).unwrap();

        assert_eq!(
            produced,
            ChatSettings {
                rate_limit: Some(RateLimitSettings {
                    tokens_per_second: 3,
                    token_bucket_size: 10,
                    slow_down_threshold: 0.8,
                })
            }
        );
    }

    #[test]
    fn slow_down_threshold_inclusive_lower_bound() {
        // The range for slow_down_threshold is inclusive, so 0.0 is a valid value
        let json = json!({
            "rate_limit": {
                "tokens_per_second": 3,
                "token_bucket_size": 10,
                "slow_down_threshold": 0
            }
        });
        let produced: ChatSettings = serde_json::from_value(json).unwrap();

        assert_eq!(
            produced,
            ChatSettings {
                rate_limit: Some(RateLimitSettings {
                    tokens_per_second: 3,
                    token_bucket_size: 10,
                    slow_down_threshold: 0.0,
                })
            }
        );
    }

    #[test]
    fn slow_down_threshold_inclusive_upper_bound() {
        // The range for slow_down_threshold is inclusive, so 1.0 is a valid value
        let json = json!({
            "rate_limit": {
                "tokens_per_second": 3,
                "token_bucket_size": 10,
                "slow_down_threshold": 1.0
            }
        });
        let produced: ChatSettings = serde_json::from_value(json).unwrap();

        assert_eq!(
            produced,
            ChatSettings {
                rate_limit: Some(RateLimitSettings {
                    tokens_per_second: 3,
                    token_bucket_size: 10,
                    slow_down_threshold: 1.0,
                })
            }
        );
    }

    #[test]
    fn deserialize_invalid_slow_down_threshold() {
        let json = json!({
            "rate_limit": {
                "tokens_per_second": 3,
                "token_bucket_size": 10,
                "slow_down_threshold": 100
            }
        });

        assert!(serde_json::from_value::<ChatSettings>(json).is_err());
    }
}
