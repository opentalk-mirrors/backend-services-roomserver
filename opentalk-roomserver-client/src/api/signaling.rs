// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::{anyhow, bail, Context as _};
use opentalk_types_common::roomserver::Token;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tungstenite::{
    protocol::{frame::coding::CloseCode, CloseFrame},
    ClientRequestBuilder, Utf8Bytes,
};
use url::Url;

pub struct SignalingConnection {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl SignalingConnection {
    pub async fn connect(roomserver_url: Url, token: Token) -> anyhow::Result<Self> {
        let mut url = roomserver_url
            .join("signaling/")
            .context("internal error, failed to append `signaling` path, invalid url")?
            .join(&token.to_string())
            .context("internal error, failed to append signaling token to path, invalid url")?;
        match url.scheme() {
            "https" => url
                .set_scheme("wss")
                .map_err(|_| anyhow!("failed to set scheme"))?,
            "http" => url
                .set_scheme("ws")
                .map_err(|_| anyhow!("failed to set scheme"))?,
            _ => bail!("unsupported url scheme"),
        }
        let uri = http::Uri::try_from(url.to_string())
            .context("internal error, failed to convert url to uri")?;

        log::debug!("connect signaling to url: {}", uri);
        let builder = ClientRequestBuilder::new(uri);
        let (socket, _response) = connect_async(builder)
            .await
            .context("failed to open signaling connection")?;

        Ok(Self { socket })
    }

    pub async fn close(mut self) -> anyhow::Result<()> {
        self.socket
            .close(Some(CloseFrame {
                code: CloseCode::Away,
                reason: Utf8Bytes::from_static(""),
            }))
            .await
            .context("Failed to close signaling socket")?;

        Ok(())
    }
}
