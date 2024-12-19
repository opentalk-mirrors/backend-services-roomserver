// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::{roomserver::Token, utils::ExampleData};
use serde::{Deserialize, Serialize};

/// The response body for GET `/rooms/{room_id}/token`
#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(TokenResponse::example_data())))]
#[serde(rename_all = "snake_case", tag = "response")]
pub enum TokenResponse {
    /// The signaling token for the requested room
    Token { token: Token },
    /// The requested room is unknown to the roomserver and needs to be provided through the [TokenRequestBody](super::TokenRequestBody)
    UnknownRoom,
}

impl ExampleData for TokenResponse {
    fn example_data() -> Self {
        Self::Token {
            token: Token::example_data(),
        }
    }
}

#[cfg(test)]
mod tests {
    use opentalk_types_common::roomserver::Token;
    use serde_json::json;

    use super::TokenResponse;

    #[test]
    fn unknown_room() {
        let value = serde_json::to_value(&TokenResponse::UnknownRoom).unwrap();

        assert_eq!(
            value,
            json!(
                {
                    "response": "unknown_room"
                }
            )
        );
    }

    #[test]
    fn token() {
        let value = serde_json::to_value(&TokenResponse::Token {
            token: Token::nil(),
        })
        .unwrap();

        assert_eq!(
            value,
            json!(
                {
                    "response": "token",
                    "token": "00000000-0000-0000-0000-000000000000"
                }
            )
        );
    }
}
