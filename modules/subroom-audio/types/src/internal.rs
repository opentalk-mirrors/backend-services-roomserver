// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_signaling::signaling_module::InternalCommand;
use opentalk_roomserver_web_api::livekit_proxy::{WebsocketRequest, WebsocketResponse};
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum SubroomAudioInternal {
    ProxyLivekitSocket {
        websocket_request: Box<WebsocketRequest>,
        return_channel: oneshot::Sender<WebsocketResponse>,
    },
}

impl InternalCommand for SubroomAudioInternal {}
