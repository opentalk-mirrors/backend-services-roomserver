// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(RateLimitSettings::example_data())))]
pub struct RateLimitSettings {
    /// The number of tokens that are added to the bucket per second
    pub tokens_per_second: u16,
    /// The maximum amount of tokens that a token bucket can hold at a time
    pub token_bucket_size: u16,
}

impl ExampleData for RateLimitSettings {
    fn example_data() -> Self {
        RateLimitSettings {
            tokens_per_second: 10,
            token_bucket_size: 50,
        }
    }
}
