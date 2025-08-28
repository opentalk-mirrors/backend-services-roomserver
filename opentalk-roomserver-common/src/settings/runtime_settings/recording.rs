// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_service_auth::ApiKey;
use url::Url;

use crate::settings::settings_file;

#[derive(Debug, Clone)]
pub struct Recording {
    pub url: Url,
    pub api_key: ApiKey,
}

impl From<settings_file::recording::Recording> for Recording {
    fn from(value: settings_file::recording::Recording) -> Self {
        Self {
            url: value.url,
            api_key: value.api_key,
        }
    }
}
