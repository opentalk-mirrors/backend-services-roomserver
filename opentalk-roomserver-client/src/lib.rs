// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! # RoomServer API Requests
//!
//! This crate provides the API requests to interact with the roomserver.
//!
//! ## Example
//!
//! ```no_run
//! # tokio_test::block_on(async {
//! # use std::{ str::FromStr as _, {collections::{BTreeMap, BTreeSet}}};
//! #
//! # use http_request_derive_client_reqwest::ReqwestClient;
//! # use http_request_derive_client::Client as _;
//! # use opentalk_roomserver_client::{
//! #     api::room::RoomsCreateRequest,
//! # };
//! # use opentalk_roomserver_types::room_parameters::RoomParameters;
//! # use opentalk_types_common::rooms::RoomId;
//! # use opentalk_types_common::utils::ExampleData;
//! #
//! let client = ReqwestClient::new("http://localhost:11333".parse().unwrap());
//! let request = RoomsCreateRequest {
//! #   room_id: RoomId::from_u128(0x8f96ada5_2660_4b4c_adb8_1b1794f51a24),
//!     body: RoomParameters {
//! // ...
//! #       ..RoomParameters::example_data()
//!     },
//! };
//!
//! let response = client.execute(request).await.unwrap();
//! println!("{:#?}", response);
//! # })
//! ```

pub mod api;
