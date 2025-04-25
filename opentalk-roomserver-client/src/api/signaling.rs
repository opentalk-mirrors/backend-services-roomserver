// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::{anyhow, Context as _};
use futures::{SinkExt as _, StreamExt};
use opentalk_types_common::roomserver::Token;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tungstenite::{
    protocol::{frame::coding::CloseCode, CloseFrame},
    ClientRequestBuilder, Message, Utf8Bytes,
};
use url::Url;

#[derive(Debug, Error)]
#[error("Signaling connection error")]
pub struct SignalingError {
    #[from]
    source: anyhow::Error,
}
pub struct SignalingConnection {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl SignalingConnection {
    pub async fn connect(roomserver_url: Url, token: Token) -> Result<Self, SignalingError> {
        let uri = build_signaling_socket_url(roomserver_url, token)?;

        log::debug!("connect signaling to url: {}", uri);
        let builder = ClientRequestBuilder::new(uri);
        let (socket, _response) = connect_async(builder)
            .await
            .context("Failed to open signaling connection")?;

        Ok(Self { socket })
    }

    pub async fn close(mut self) -> Result<(), SignalingError> {
        self.socket
            .close(Some(CloseFrame {
                code: CloseCode::Away,
                reason: Utf8Bytes::from_static(""),
            }))
            .await
            .context("Failed to close signaling socket")?;

        Ok(())
    }

    pub async fn send_raw_message(&mut self, message: &str) -> Result<(), SignalingError> {
        log::debug!("send text message: {:?}", message);
        self.socket
            .send(Message::Text(message.into()))
            .await
            .context("Failed to send message")?;

        Ok(())
    }

    pub async fn receive_raw_message(&mut self) -> Result<Option<String>, SignalingError> {
        let Some(msg) = self.socket.next().await else {
            return Ok(None);
        };
        let msg = msg.context("receive error")?;

        log::debug!("received message: {:?}", msg);

        match msg {
            Message::Text(utf8_bytes) => Ok(Some(utf8_bytes.to_string())),

            // don't log the full message, just the type
            Message::Binary(_) => Err(anyhow!("Expected text messsage, got: Binary").into()),
            Message::Ping(_) => Err(anyhow!("Expected text messsage, got: Ping").into()),
            Message::Pong(_) => Err(anyhow!("Expected text messsage, got: Pong").into()),
            Message::Close(_) => Ok(None),
            Message::Frame(_) => Err(anyhow!("Expected text messsage, got: Frame").into()),
        }
    }
}

fn build_signaling_socket_url(roomserver_url: Url, token: Token) -> anyhow::Result<http::Uri> {
    let mut url = roomserver_url
        .join("signaling/")
        .context("Internal error, failed to append `signaling` path, invalid url")?
        .join(&token.to_string())
        .context("Internal error, failed to append signaling token to path, invalid url")?;
    match url.scheme() {
        "https" => url
            .set_scheme("wss")
            .map_err(|_| anyhow!("Failed to set scheme"))?,
        _ => url
            .set_scheme("ws")
            .map_err(|_| anyhow!("Failed to set scheme"))?,
    }
    let uri = http::Uri::try_from(url.to_string())
        .context("Internal error, failed to convert url to uri")?;
    Ok(uri)
}
