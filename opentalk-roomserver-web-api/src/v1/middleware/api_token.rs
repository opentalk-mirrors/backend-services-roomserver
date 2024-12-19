// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::future::{ready, Ready};

use axum::{extract::Request, http::header::AUTHORIZATION, response::IntoResponse};
use futures::future::Either;
use opentalk_types_api_v1::error::ApiError;
use tower::{Layer, Service};

#[derive(Debug, Clone)]
pub(crate) struct ServiceAuthLayer {
    pub api_token: String,
}

impl<S> Layer<S> for ServiceAuthLayer {
    type Service = ApiTokenMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ApiTokenMiddleware {
            inner,
            api_token: self.api_token.clone(),
        }
    }
}

/// Checks the provided API token in the authorization header of the request
///
/// The API token is expected to be in the `Authorization` header with the format `bearer <token>`.
///
/// Returns a `401` Unauthorized error when the provided token is invalid or missing
/// Returns a `400` Bad Request error when the authorization header could not be parsed due to invalid ascii or formatting
#[derive(Debug, Clone)]
pub(crate) struct ApiTokenMiddleware<S> {
    inner: S,
    api_token: String,
}

impl<S> Service<Request> for ApiTokenMiddleware<S>
where
    S: Service<Request, Response = axum::response::Response>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let Some(header) = req.headers().get(AUTHORIZATION) else {
            return Either::Right(ready(Ok(ApiError::unauthorized()
                .with_code("missing_api_token")
                .with_message("missing API-Token in authorization header")
                .into_response())));
        };

        let Ok(auth_header) = header.to_str() else {
            return Either::Right(ready(Ok(ApiError::bad_request()
                .with_code("invalid_authorization_header")
                .with_message("failed to parse API-Token in authorization header")
                .into_response())));
        };

        let token = match auth_header.trim().split_once(' ') {
            Some(("bearer", token)) => token.trim(),
            Some(("Bearer", token)) => token.trim(),
            _ => {
                return Either::Right(ready(Ok(ApiError::bad_request()
                    .with_code("invalid_authorization_header")
                    .with_message("failed to parse API-Token in authorization header")
                    .into_response())));
            }
        };

        if token != self.api_token {
            return Either::Right(ready(Ok(ApiError::unauthorized()
                .with_code("invalid_api_token")
                .with_message("invalid API-Token in authorization header")
                .into_response())));
        }

        Either::Left(self.inner.call(req))
    }
}
