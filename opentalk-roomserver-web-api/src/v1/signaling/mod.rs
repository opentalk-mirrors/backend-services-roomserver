// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! This module provides two way communication between the room server and it's clients (e.g. web-app).

use std::fmt::Debug;

use axum::{
    async_trait,
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::{IntoResponse, Response},
    routing::get,
};
use opentalk_types_common::rooms::RoomId;

use super::Router;

pub mod websocket;

pub(crate) fn routes<B: SignalingBackend + 'static>() -> Router<B> {
    Router::new().route("/signaling/:room_id", get(handler::<B>))
}

#[utoipa::path(
    get,
    path = "/signaling/{room_id}",
    responses(
        (status = StatusCode::SWITCHING_PROTOCOLS, description = "Successfully upgraded connection to WebSocket"),
        (status = StatusCode::NOT_FOUND, description = "The requested room does not exist"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "An internal server error occurred"),
    ),
    params(
        ("room_id" = RoomId, Path, description = "The UUID that identifies the room")
    ),
    security(),
    )]
#[tracing::instrument(name = "/signaling/{room_id}", level = "info", skip(ctx, ws))]
async fn handler<B: SignalingBackend + 'static>(
    State(ctx): State<B>,
    Path(room_id): Path<RoomId>,
    ws: WebSocketUpgrade,
) -> Result<Response, B::Error> {
    log::debug!("Received signaling connection request");

    // This refreshes the rooms idle timeout to avoid race conditions
    ctx.ensure_room_exists(room_id).await?;

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, ctx, room_id)))
}

async fn handle_socket<B: SignalingBackend + 'static>(socket: WebSocket, ctx: B, room_id: RoomId) {
    log::debug!("Upgrade to websocket connection");
    if let Err(e) = ctx.accept_client_stream(socket, room_id).await {
        log::info!("Failed to accept client stream: {e:?}");
    }
}

/// Logic for handling signaling steams.
///
/// Before a new stream can be added, the WebAPI ensures that the room to which
/// the signaling stream should be added exists using [`ensure_room_exists`](SignalingBackend::ensure_room_exists).
#[async_trait]
pub trait SignalingBackend: Clone + Send + Sync + std::fmt::Debug {
    type Error: Debug + IntoResponse;

    /// Returns an error if the room doesn't exist.
    async fn ensure_room_exists(&self, room_id: RoomId) -> Result<(), Self::Error>;

    /// Accept a client stream and connect it to the room.
    ///
    /// # Error
    ///
    /// If this function errors, no response will be send to the user. The
    /// implementation must ensure to close the socket.
    async fn accept_client_stream(
        &self,
        socket: WebSocket,
        room_id: RoomId,
    ) -> Result<(), Self::Error>;
}
