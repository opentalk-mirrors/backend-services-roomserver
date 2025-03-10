// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! # RoomServer Client
//!
//! This crate implements a room server client library that can be used to make
//! API calls to the room server.
//!
//! The [`Client`] trait describes the interface to make requests to the room server.
//! Currently only the HTTP client [`reqwest_client::RoomServerClient`] exists.
//!
//! Requests are constructed using the request structs found in [`api`] and executed
//! using an implementation of the [`Client`] trait.
//!
//! ## Example
//!
//! ```no_run
//! # tokio_test::block_on(async {
//! # use std::{ str::FromStr as _, {collections::{BTreeMap, BTreeSet}}};
//! #
//! # use opentalk_roomserver_client::{
//! #     api::room::RoomsCreateRequest, reqwest_client::RoomServerClient, Client,
//! # };
//! # use opentalk_roomserver_types::room_parameters::{EventContext, RoomParameters};
//! # use opentalk_types_common::{
//! #     rooms::RoomId,
//! #     tariffs::{TariffId, TariffResource},
//! #     users::UserId,
//! # };
//! # use opentalk_types_api_v1::users::PublicUserProfile;
//! # use opentalk_types_common::users::{UserTitle, DisplayName};
//! # use opentalk_types_common::events::{EventTitle, EventDescription};
//! #
//! let client = RoomServerClient::new("http://localhost:11333").unwrap();
//! let request = RoomsCreateRequest {
//! #   room_id: RoomId::from_u128(0x8f96ada5_2660_4b4c_adb8_1b1794f51a24),
//!     body: RoomParameters {
//! // ...
//! #        created_by: PublicUserProfile {
//! #            id: UserId::from_u128(0x037bc784_5130_4da7_b63f_971395be0e44),
//! #            email: "peter@example.net".to_owned(),
//! #            title: UserTitle::from_str("Prof. Dr. Dr. Dipl. Ing.").unwrap(),
//! #            firstname: "Peter".to_owned(),
//! #            lastname: "Superschlau".to_owned(),
//! #            display_name: DisplayName::from_str("Prof. Dr. Dr. Dipl. Ing. Superschlau").unwrap(),
//! #            avatar_url: "example.com".to_owned(),
//! #        },
//! #        password: Some("supersecret".to_owned()),
//! #        waiting_room: false,
//! #        call_in: None,
//! #        event: Some(EventContext {
//! #            title: EventTitle::from_str("Example Event").unwrap(),
//! #            description: EventDescription::from_str("An example event.").unwrap(),
//! #            is_adhoc: false,
//! #            shared_folder: None,
//! #        }),
//! #        invite_code: None,
//! #        tariff: TariffResource {
//! #            id: TariffId::from_u128(0x35499437_32a3_4b30_87cc_568eaf63ed9e),
//! #            name: "SuperPremium".to_owned(),
//! #            quotas: BTreeMap::new(),
//! #            modules: BTreeMap::new(),
//! #        },
//! #        streaming_links: Vec::new(),
//!     },
//! };
//!
//! let response = client.execute(request).await.unwrap();
//! println!("{:#?}", response);
//! # })
//! ```

pub mod api;
pub mod reqwest_client;

use http_request_derive::HttpRequest;

/// Possible error kinds that can occur when using the room server client and
/// executing requests.
pub enum ErrorKind {
    /// The api call was not authorized.
    Authorization,

    /// The resource was not found.
    NotFound,

    /// An error that was caused by the client.
    ClientError,

    /// An error that was caused by the server.
    ServerError,

    /// An unknown error
    Unknown,
}

/// Get the [`ErrorKind`] for a specific error.
pub trait GetErrorKind {
    fn get_kind(&self) -> ErrorKind;
}

/// A client that can execute room server api requests.
///
/// This is implemented by [`reqwest_client::RoomServerClient`], which queries
/// the room server using HTTP.
#[async_trait::async_trait]
pub trait Client {
    type Error: std::error::Error + GetErrorKind + Send + Sync;

    /// Execute a request and parse the response from the room server.
    ///
    /// ## Error
    ///
    /// Errors that occur during execution implement the [`GetErrorKind`] trait.
    /// If a specific error is expected the [`ErrorKind`] can be used to match
    /// against that variant.
    async fn execute<R: HttpRequest + Send>(&self, request: R) -> Result<R::Response, Self::Error>;
}
