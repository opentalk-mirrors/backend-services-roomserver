// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! # HTTP Room Server Client (using [reqwest](https://docs.rs/reqwest/latest/reqwest/))
//!
//! [`RoomServerClient`] implements the [`Client`] trait and calls the room server
//! using the HTTP api.

use http_request_derive::HttpRequest;
use url::{ParseError, Url};

use crate::{Client, GetErrorKind};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

fn default_user_agent() -> String {
    format!("{NAME}/{VERSION}")
}

/// Errors that occur during initialization of the room server client.
#[derive(thiserror::Error, Debug)]
pub enum SetupError {
    #[error("not a valid url")]
    MalformedUrl(#[from] ParseError),

    #[error("the url must not contain the component `{component}`")]
    UnsupportedUrlComponent { component: String },
}

/// Errors that occur while executing a request.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Not authorized to access the requested resource
    #[error("unauthorized")]
    Unauthorized,

    /// An error that was caused by a fault in the room server client
    #[error("internal client error: {source}")]
    Internal { source: http_request_derive::Error },

    /// An error that occurred during the request to the server (e.g. timeout, connection loss etc.)
    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),

    /// An error that was caused by a fault in server
    #[error("server error: {source}")]
    Server { source: http_request_derive::Error },

    /// An error that was caused by a fault in server
    #[error("{msg}: {source}")]
    Custom {
        msg: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl From<http_request_derive::Error> for Error {
    fn from(value: http_request_derive::Error) -> Self {
        match value {
            http_request_derive::Error::Unauthorized => Self::Unauthorized,
            http_request_derive::Error::NonSuccessStatus { .. } => Self::Server { source: value },
            http_request_derive::Error::BuildRequest { .. }
            | http_request_derive::Error::UrlCannotBeABase { .. }
            | http_request_derive::Error::ParseUri { .. }
            | http_request_derive::Error::QueryString { .. }
            | http_request_derive::Error::SerdeUrlParams { .. }
            | http_request_derive::Error::Reqwest { .. }
            | http_request_derive::Error::Json { .. } => Self::Internal { source: value },
            http_request_derive::Error::Custom { .. } => Self::Custom {
                msg: "Error in http_request_derive".to_string(),
                source: Box::new(value),
            },
        }
    }
}

impl GetErrorKind for Error {
    fn get_kind(&self) -> crate::ErrorKind {
        match self {
            Error::Unauthorized => crate::ErrorKind::Authorization,
            Error::Internal { .. } => crate::ErrorKind::ClientError,
            Error::Request { .. } | Error::Server { .. } => crate::ErrorKind::ServerError,
            Error::Custom { .. } => crate::ErrorKind::ClientError,
        }
    }
}

/// Room Server client using the HTTP API of the room server.
///
/// The room server client send use a user agent string using the package name
/// and version. This behavior can be customized by providing a [`reqwest::Client`]
/// and using [`RoomServerClient::with_reqwest_client`].
///
/// Requests can be executed using [`RoomServerClient::execute`].
pub struct RoomServerClient {
    client: reqwest::Client,
    base_url: Url,
}

impl RoomServerClient {
    /// Initialize the room server client.
    ///
    /// ```
    /// use opentalk_roomserver_client::reqwest_client::RoomServerClient;
    ///
    /// let client = RoomServerClient::new("https://localhost:11311");
    /// ```
    pub fn new(addr: &str) -> Result<Self, SetupError> {
        let client = reqwest::Client::builder()
            .user_agent(default_user_agent())
            .build()
            .expect("User agent must be valid.");

        Self::with_reqwest_client(addr, client)
    }

    /// Initialize the room server client with a pre-configured reqwest client.
    ///
    /// ```
    /// use opentalk_roomserver_client::reqwest_client::RoomServerClient;
    /// use reqwest::{header, Client};
    ///
    /// let mut headers = header::HeaderMap::new();
    /// headers.insert("X-MY-HEADER", header::HeaderValue::from_static("value"));
    ///
    /// let reqwest_client = Client::builder()
    ///     .default_headers(headers)
    ///     .build()
    ///     .unwrap();
    /// let client = RoomServerClient::with_reqwest_client("https://localhost:11311", reqwest_client);
    /// ```
    pub fn with_reqwest_client(addr: &str, client: reqwest::Client) -> Result<Self, SetupError> {
        let base_url = Url::parse(addr)?;

        if base_url.fragment().is_some() {
            return Err(SetupError::UnsupportedUrlComponent {
                component: "fragment".to_owned(),
            });
        }

        if base_url.query().is_some() {
            return Err(SetupError::UnsupportedUrlComponent {
                component: "query parameter".to_owned(),
            });
        }

        if base_url.path() != "/" {
            return Err(SetupError::UnsupportedUrlComponent {
                component: "path".to_owned(),
            });
        }

        Ok(Self { base_url, client })
    }
}

#[async_trait::async_trait]
impl Client for RoomServerClient {
    type Error = Error;

    async fn execute<R: HttpRequest + Send>(&self, request: R) -> Result<R::Response, Self::Error> {
        let http_request = request.to_http_request(&self.base_url)?;

        let response = self
            .client
            .execute(http_request.try_into().map_err(|err| Error::Custom {
                msg: "failed to convert http::Request to reqwest::Request".to_string(),
                source: Box::new(err),
            })?)
            .await?;

        let response = R::read_reqwest_response(response).await?;
        Ok(response)
    }
}
