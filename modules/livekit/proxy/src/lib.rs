// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::{anyhow, bail};
use futures::{SinkExt as _, StreamExt as _};
use opentalk_roomserver_types::livekit_proxy::{
    LiveKitAccessToken, PreparedSocket,
    websocket::{CloseFrame, LiveKitSocket, LiveKitSocketMessage},
};
use tokio::sync::oneshot;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        Message as TungsteniteMessage,
        client::IntoClientRequest,
        http::{HeaderValue, header::AUTHORIZATION},
        protocol::{CloseFrame as TungsteniteCloseFrame, frame::coding::CloseCode},
    },
};
use url::Url;

/// A wrapper around `oneshot::Sender<()>` that sends on drop,
/// ensuring the shutdown signal is always delivered.
#[derive(Debug)]
pub struct ShutdownSender(Option<oneshot::Sender<()>>);

impl ShutdownSender {
    pub fn new() -> (Self, oneshot::Receiver<()>) {
        let (tx, rx) = oneshot::channel();
        (Self(Some(tx)), rx)
    }
}

impl Drop for ShutdownSender {
    fn drop(&mut self) {
        if let Some(tx) = self.0.take() {
            let _ = tx.send(());
        }
    }
}

/// Connects to the upstream LiveKit server and returns the WebSocket connection.
pub async fn connect_to_livekit(
    livekit_rtc_url: Url,
    access_token: LiveKitAccessToken,
) -> anyhow::Result<PreparedSocket> {
    let request = match access_token {
        LiveKitAccessToken::Header(token) => {
            let mut request = livekit_rtc_url
                .as_str()
                .into_client_request()
                .map_err(|err| anyhow!("failed to build livekit websocket request: {err}"))?;

            let header_value = HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|err| anyhow!("failed to build livekit authorization header: {err}"))?;
            request.headers_mut().insert(AUTHORIZATION, header_value);
            request
        }
        LiveKitAccessToken::Query(token) => {
            let livekit_rtc_url = livekit_rtc_url.to_string();
            let separator = if livekit_rtc_url.contains('?') {
                '&'
            } else {
                '?'
            };
            let uri = format!("{livekit_rtc_url}{separator}access_token={token}");
            uri.into_client_request()
                .map_err(|err| anyhow!("failed to build livekit websocket request: {err}"))?
        }
    };

    let (upstream_socket, _) = connect_async(request)
        .await
        .map_err(|err| anyhow!("failed to connect to livekit websocket: {err}"))?;

    Ok(upstream_socket)
}

pub async fn proxy_websocket(
    upstream_socket: PreparedSocket,
    downstream_socket: Box<dyn LiveKitSocket>,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    let (mut upstream_sink, mut upstream_stream) = upstream_socket.split();
    let (mut downstream_sink, mut downstream_stream) = downstream_socket.split();

    loop {
        tokio::select! {
            _ = &mut shutdown_rx => {
                break;
            }
            item = downstream_stream.next() => {
                let Some(item) = item else {
                    break;
                };

                let item = match item {
                    Ok(item) => item,
                    Err(err) => {
                        tracing::debug!("error reading downstream livekit socket: {err:#}");
                        break;
                    }
                };

                let Some(message) = map_downstream_message(item) else {
                    continue;
                };
                let is_close = matches!(message, TungsteniteMessage::Close(_));

                if upstream_sink.send(message).await.is_err() {
                    break;
                }


                if is_close {
                    break;
                }
            }
            item = upstream_stream.next() => {
                let Some(item) = item else {
                    break;
                };

                let message = match item {
                    Ok(message) => message,
                    Err(err) => {
                        tracing::debug!("error reading upstream livekit socket: {err:#}");
                        break;
                    }
                };

                let Some(message) = map_upstream_message(message) else {
                    continue;
                };

                let is_close = matches!(message, LiveKitSocketMessage::Close(_));
                if downstream_sink.send(message).await.is_err() {
                    break;
                }

                if is_close {
                    break;
                }
            }
        }
    }

    Ok(())
}

pub fn build_livekit_rtc_url(service_url: &Url) -> Result<Url, anyhow::Error> {
    let mut url = service_url.clone();
    if url.scheme() == "https" {
        url.set_scheme("wss")
            .map_err(|_| anyhow!("Invalid scheme"))?;
    } else if url.scheme() == "http" {
        url.set_scheme("ws")
            .map_err(|_| anyhow!("Invalid scheme"))?;
    } else {
        bail!("unsupported scheme");
    }

    // Do not use join as we want to not replace the last segment if a trailing
    // slash is missing.
    url.path_segments_mut()
        .map_err(|_| anyhow!("Invalid URL cannot be base"))?
        .push("rtc");

    Ok(url)
}

fn map_downstream_message(message: LiveKitSocketMessage) -> Option<TungsteniteMessage> {
    match message {
        LiveKitSocketMessage::Text(text) => Some(TungsteniteMessage::Text(text.into())),
        LiveKitSocketMessage::Binary(binary) => Some(TungsteniteMessage::Binary(binary)),
        LiveKitSocketMessage::Ping(bytes) => Some(TungsteniteMessage::Ping(bytes)),
        LiveKitSocketMessage::Pong(_) => None,
        LiveKitSocketMessage::Close(close_frame) => {
            Some(TungsteniteMessage::Close(close_frame.map(|frame| {
                TungsteniteCloseFrame {
                    code: CloseCode::from(frame.code),
                    reason: frame.reason.into(),
                }
            })))
        }
    }
}

fn map_upstream_message(message: TungsteniteMessage) -> Option<LiveKitSocketMessage> {
    match message {
        TungsteniteMessage::Text(text) => Some(LiveKitSocketMessage::Text(text.to_string())),
        TungsteniteMessage::Binary(binary) => Some(LiveKitSocketMessage::Binary(binary)),
        TungsteniteMessage::Ping(_) | TungsteniteMessage::Pong(_) => None,
        TungsteniteMessage::Close(close_frame) => {
            Some(LiveKitSocketMessage::Close(close_frame.map(|frame| {
                CloseFrame {
                    code: frame.code.into(),
                    reason: frame.reason.to_string(),
                }
            })))
        }
        TungsteniteMessage::Frame(_) => None,
    }
}
