// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::{roomserver::Token, utils::ExampleData};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(RoomServerAccess::example_data())))]
pub struct RoomServerAccess {
    /// The public url for the roomserver with the requested room
    pub public_url: Url,
    /// The signaling token for the requested room
    pub token: Token,
}

impl ExampleData for RoomServerAccess {
    fn example_data() -> Self {
        Self {
            token: Token::example_data(),
            public_url: Url::parse("https://example.com").unwrap(),
        }
    }
}

/// The response body for GET `/rooms/{room_id}/token`
#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(TokenResponse::example_data())))]
#[serde(rename_all = "snake_case", tag = "response")]
pub enum TokenResponse {
    /// The signaling token for the requested room
    Token(RoomServerAccess),
    /// The requested room is unknown to the roomserver and needs to be provided through the
    /// [TokenRequestBody](super::TokenRequestBody)
    UnknownRoom,
}

impl ExampleData for TokenResponse {
    fn example_data() -> Self {
        Self::Token(RoomServerAccess::example_data())
    }
}

#[cfg(test)]
mod tests {
    use opentalk_types_common::roomserver::Token;
    use serde_json::json;
    use url::Url;

    use super::TokenResponse;
    use crate::api::token_response::RoomServerAccess;

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
        let value = serde_json::to_value(TokenResponse::Token(RoomServerAccess {
            token: Token::nil(),
            public_url: Url::parse("https://example.com/").unwrap(),
        }))
        .unwrap();

        assert_eq!(
            value,
            json!(
                {
                    "response": "token",
                    "token": "00000000-0000-0000-0000-000000000000",
                    "public_url": "https://example.com/",
                }
            )
        );
    }
}
