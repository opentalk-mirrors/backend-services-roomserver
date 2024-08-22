// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

pub mod v1;

pub(crate) type Router<Logic> = axum::Router<Logic>;
