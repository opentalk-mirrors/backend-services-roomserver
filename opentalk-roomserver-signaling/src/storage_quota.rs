// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

pub struct StorageQuota(u64);

impl From<u64> for StorageQuota {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<StorageQuota> for u64 {
    fn from(value: StorageQuota) -> Self {
        value.0
    }
}
