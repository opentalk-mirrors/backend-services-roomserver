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
