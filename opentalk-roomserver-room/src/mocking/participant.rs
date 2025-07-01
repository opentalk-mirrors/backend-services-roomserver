// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{str::FromStr, time::Duration};

use axum::extract::ws::Message;
use opentalk_roomserver_signaling::{
    breakout::BREAKOUT_MODULE_ID, signaling_event::SignalingEvent,
    signaling_module::SignalingModule,
};
use opentalk_roomserver_types::{
    breakout::{breakout_config::BreakoutConfig, command::BreakoutCommand, event::BreakoutEvent},
    client_parameters::{ClientKind, ClientParameters, Role},
    connection_id::ConnectionId,
    core_event::CoreEvent,
    join::join_success::JoinSuccess,
    room_kind::RoomKind,
    signaling::SignalingCommand,
};
use opentalk_roomserver_web_api::v1::signaling::websocket;
use opentalk_types_api_v1::users::PublicUserProfile;
use opentalk_types_common::{
    roomserver::DeviceSecret,
    users::{DisplayName, UserId, UserInfo},
};
use opentalk_types_signaling::ParticipantId;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{json, value::to_raw_value};
use tokio::{
    sync::mpsc,
    time::{error::Elapsed, timeout},
};

use super::{
    room::{self, TestRoom},
    socket::MockSocket,
};

const RECV_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug)]
pub enum ReceiveError {
    Closed,
    Timeout,
    InvalidJson {
        error: serde_json::Error,
        message: Message,
    },
    UnexpectedMessage(Message),
}

#[derive(Debug)]
pub enum SendError {
    Closed,
    InvalidJson(serde_json::Error),
    UnexpectedMessage(Message),
}

impl<T> From<mpsc::error::SendError<T>> for SendError {
    fn from(_: mpsc::error::SendError<T>) -> Self {
        SendError::Closed
    }
}

impl From<Elapsed> for ReceiveError {
    fn from(_value: Elapsed) -> Self {
        Self::Timeout
    }
}

pub type MockParticipantJoining = MockParticipant<()>;
pub type MockParticipantJoined = MockParticipant<JoinSuccess>;

#[derive(Debug)]
pub struct MockParticipant<S> {
    pub(crate) sender: mpsc::Sender<Result<Message, websocket::Error>>,
    pub(crate) receiver: mpsc::Receiver<Message>,
    pub(crate) state: S,
}

impl MockParticipant<()> {
    pub(crate) async fn join_success(
        mut self,
    ) -> Result<MockParticipant<JoinSuccess>, ReceiveError> {
        let Some(received) = self.receiver.recv().await else {
            return Err(ReceiveError::Closed);
        };
        match received {
            Message::Text(text) => {
                let event: SignalingEvent<CoreEvent> =
                    serde_json::from_str(&text).map_err(|error| ReceiveError::InvalidJson {
                        error,
                        message: Message::Text(text.clone()),
                    })?;

                if let CoreEvent::JoinSuccess(msg) = event.content {
                    Ok(MockParticipant {
                        sender: self.sender,
                        receiver: self.receiver,
                        state: *msg,
                    })
                } else {
                    Err(ReceiveError::UnexpectedMessage(Message::Text(text)))
                }
            }
            other => Err(ReceiveError::UnexpectedMessage(other)),
        }
    }
}

impl MockParticipant<JoinSuccess> {
    pub fn join_success(&self) -> &JoinSuccess {
        &self.state
    }

    pub fn id(&self) -> ParticipantId {
        self.state.id
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.state.connection_id
    }

    pub async fn start_breakout_rooms(
        &mut self,
        others: &mut [&mut MockParticipantJoined],
        config: BreakoutConfig,
    ) -> BreakoutEvent {
        self.send_breakout_command(BreakoutCommand::Start(config), None)
            .await
            .unwrap();

        for p in others {
            assert!(matches!(
                p.receive::<BreakoutEvent>().await.unwrap().content,
                BreakoutEvent::Started { .. },
            ));
        }

        self.receive::<BreakoutEvent>().await.unwrap().content
    }

    pub async fn switch_breakout_room(
        &mut self,
        others: &mut [&mut MockParticipantJoined],
        room: RoomKind,
    ) -> BreakoutEvent {
        self.send_breakout_command(BreakoutCommand::SwitchRoom(room), None)
            .await
            .unwrap();

        for p in others {
            assert!(matches!(
                p.receive::<BreakoutEvent>().await.unwrap().content,
                BreakoutEvent::ParticipantSwitchedRoom { .. },
            ));
        }

        self.receive::<BreakoutEvent>().await.unwrap().content
    }

    pub async fn stop_breakout_rooms(
        &mut self,
        others: &mut [&mut MockParticipantJoined],
    ) -> BreakoutEvent {
        self.send_breakout_command(BreakoutCommand::Stop { delay: None }, None)
            .await
            .unwrap();

        for p in others.iter_mut() {
            let event = p.receive::<BreakoutEvent>().await.unwrap().content;
            assert!(
                matches!(event, BreakoutEvent::Closing { .. }),
                "Expected Closing notice, got: {event:?}"
            );
        }

        for p in others.iter_mut() {
            let event = p.receive::<BreakoutEvent>().await.unwrap().content;
            assert!(
                matches!(event, BreakoutEvent::Closed),
                "Expected Closed notice, got: {event:?}"
            );
        }

        self.receive::<BreakoutEvent>().await.unwrap().content
    }
}

impl<S> MockParticipant<S> {
    pub fn alice(device_number: usize) -> MockParticipantBuilder<PublicUserProfile> {
        let profile = PublicUserProfile {
            id: UserId::from_u128(0xa11ce),
            email: "alice@example.com".to_string(),
            user_info: UserInfo {
                title: "M.Sc.".parse().expect("Valid title"),
                firstname: "Alice".to_string(),
                lastname: "Aal".to_string(),
                display_name: "Alice the angry".parse().expect("Valid DisplayName"),
                avatar_url: "https://example.com/avatar-of-alice".to_string(),
            },
        };

        MockParticipantBuilder {
            profile,
            role: Role::Moderator,
            secret: DeviceSecret::from_str(&format!("Alice Device Secret {device_number}"))
                .expect("Valid device secret"),
        }
    }

    pub fn bob(device_number: usize) -> MockParticipantBuilder<PublicUserProfile> {
        let profile = PublicUserProfile {
            id: UserId::from_u128(0xb0b),
            email: "bob@example.com".to_string(),
            user_info: UserInfo {
                title: "".parse().expect("Valid title"),
                firstname: "Bob".to_string(),
                lastname: "Barsch".to_string(),
                display_name: "Bob the bold".parse().expect("Valid DisplayName"),
                avatar_url: "https://example.com/avatar-of-bob".to_string(),
            },
        };

        MockParticipantBuilder {
            profile,
            role: Role::User,
            secret: DeviceSecret::from_str(&format!("Bob Device Secret {device_number}"))
                .expect("Valid device secret"),
        }
    }

    pub fn charlie(device_number: usize) -> MockParticipantBuilder<PublicUserProfile> {
        let profile = PublicUserProfile {
            id: UserId::from_u128(0xcca211e),
            email: "charlie@example.com".to_string(),
            user_info: UserInfo {
                title: "".parse().expect("Valid title"),
                firstname: "Charlie".to_string(),
                lastname: "Clownfisch".to_string(),
                display_name: "Charlie the charming".parse().expect("Valid DisplayName"),
                avatar_url: "https://example.com/avatar-of-alice".to_string(),
            },
        };

        MockParticipantBuilder {
            profile,
            role: Role::User,
            secret: DeviceSecret::from_str(&format!("Charlie Device Secret {device_number}"))
                .expect("Valid device secret"),
        }
    }

    pub fn gustav() -> MockParticipantBuilder<DisplayName> {
        MockParticipantBuilder {
            profile: "Gustav the great".parse().expect("Valid DisplayName"),
            role: Role::User,
            secret: DeviceSecret::from_str("Gustav Device Secret A").expect("Valid device secret"),
        }
    }

    async fn static_send_ping(
        sender: &mut mpsc::Sender<Result<Message, websocket::Error>>,
    ) -> Result<(), SendError> {
        sender
            .send(Ok(axum::extract::ws::Message::Text(
                json!( {
                    "namespace": "ping",
                    "content": serde_json::Value::Null,
                })
                .to_string()
                .into(),
            )))
            .await?;
        Ok(())
    }

    pub fn queue_send_ping(&self) {
        let mut sender = self.sender.clone();
        tokio::spawn(async move {
            Self::static_send_ping(&mut sender)
                .await
                .expect("Send should succeed eventually");
        });
    }

    pub async fn send_command<M>(
        &self,
        command: M::Incoming,
        transaction_id: Option<u64>,
    ) -> Result<(), SendError>
    where
        M: SignalingModule,
        M::Incoming: Serialize,
    {
        let command = SignalingCommand {
            namespace: M::NAMESPACE,
            transaction_id,
            content: to_raw_value(&command).expect("Command must be Serializable"),
        };
        let value = serde_json::to_value(&command).expect("SignalingCommand is serializable");
        self.send_command_raw(value).await
    }

    pub async fn send_command_raw(
        &self,
        command: serde_json::value::Value,
    ) -> Result<(), SendError> {
        self.sender
            .send(Ok(Message::Text(command.to_string().into())))
            .await?;
        Ok(())
    }

    pub async fn send_breakout_command(
        &self,
        command: BreakoutCommand,
        transaction_id: Option<u64>,
    ) -> Result<(), SendError> {
        let command = SignalingCommand {
            namespace: BREAKOUT_MODULE_ID,
            transaction_id,
            content: to_raw_value(&command).expect("Command must be serializable"),
        };
        let value = serde_json::to_value(&command).expect("BreakoutCommand is serializable");
        self.send_command_raw(value).await
    }

    pub async fn receive_event<M>(&mut self) -> Result<SignalingEvent<M::Outgoing>, ReceiveError>
    where
        M: SignalingModule,
        M::Outgoing: DeserializeOwned,
    {
        let Some(received) = timeout(RECV_TIMEOUT, self.receiver.recv()).await? else {
            return Err(ReceiveError::Closed);
        };
        match received {
            Message::Text(text) => {
                let event: SignalingEvent<M::Outgoing> =
                    serde_json::from_str(&text).map_err(|error| ReceiveError::InvalidJson {
                        error,
                        message: Message::Text(text),
                    })?;
                Ok(event)
            }
            other => Err(ReceiveError::UnexpectedMessage(other)),
        }
    }

    pub async fn receive<E: DeserializeOwned>(
        &mut self,
    ) -> Result<SignalingEvent<E>, ReceiveError> {
        let Some(received) = timeout(RECV_TIMEOUT, self.receiver.recv()).await? else {
            return Err(ReceiveError::Closed);
        };
        match received {
            Message::Text(text) => {
                let event: SignalingEvent<E> =
                    serde_json::from_str(&text).map_err(|error| ReceiveError::InvalidJson {
                        error,
                        message: Message::Text(text),
                    })?;

                Ok(event)
            }
            other => Err(ReceiveError::UnexpectedMessage(other)),
        }
    }

    pub fn received_nothing(&mut self) -> bool {
        self.receiver.is_empty()
    }

    pub fn disconnect(self) {
        // Dropping the sender will result in a disconnect
        drop(self.sender);
    }
}

pub(crate) fn create_participant_connection() -> (MockSocket, MockParticipantJoining) {
    let websocket_in = mpsc::channel(1);
    let websocket_out = mpsc::channel(1);
    (
        MockSocket::new(websocket_in.1, websocket_out.0),
        MockParticipantJoining {
            sender: websocket_in.0,
            receiver: websocket_out.1,
            state: (),
        },
    )
}

pub struct MockParticipantBuilder<P> {
    profile: P,
    role: Role,
    secret: DeviceSecret,
}

impl<P> MockParticipantBuilder<P> {
    pub fn secret(mut self, secret: DeviceSecret) -> Self {
        self.secret = secret;
        self
    }

    pub fn moderator(mut self) -> Self {
        self.role = Role::Moderator;
        self
    }

    pub fn user(mut self) -> Self {
        self.role = Role::User;
        self
    }
}

impl MockParticipantBuilder<PublicUserProfile> {
    pub fn id(mut self, id: UserId) -> Self {
        self.profile.id = id;
        self
    }

    pub fn email(mut self, email: String) -> Self {
        self.profile.email = email;
        self
    }

    /// Panics if the title is invalid
    pub fn title(mut self, title: &str) -> Self {
        self.profile.user_info.title = title.parse().unwrap();
        self
    }

    pub fn firstname(mut self, firstname: String) -> Self {
        self.profile.user_info.firstname = firstname;
        self
    }

    pub fn lastname(mut self, lastname: String) -> Self {
        self.profile.user_info.lastname = lastname;
        self
    }

    /// Panics if the display name is invalid
    pub fn display_name(mut self, display_name: String) -> Self {
        self.profile.user_info.display_name = display_name.parse().unwrap();
        self
    }

    pub async fn join(
        self,
        room: &mut TestRoom,
    ) -> Result<MockParticipant<JoinSuccess>, room::Error> {
        room.join_participant(ClientParameters {
            device_secret: self.secret,
            kind: ClientKind::Registered {
                profile: self.profile,
            },
            role: self.role,
        })
        .await
    }
}

// Guest builder
impl MockParticipantBuilder<DisplayName> {
    /// Panics if the display name is invalid
    pub fn display_name(mut self, display_name: String) -> Self {
        self.profile = display_name.parse().unwrap();
        self
    }

    pub async fn join(
        self,
        room: &mut TestRoom,
    ) -> Result<MockParticipant<JoinSuccess>, room::Error> {
        room.join_participant(ClientParameters {
            device_secret: self.secret,
            kind: ClientKind::Guest {
                display_name: self.profile,
            },
            role: self.role,
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::mocking::{
        mock_module::{MockCommand, MockModule},
        room::TestRoom,
    };

    #[test_log::test(tokio::test)]
    async fn received_nothing() {
        let mut room = TestRoom::builder().register_module::<MockModule>().spawn();
        let mut alice = room.join_alice_moderator(0).await;

        alice
            .send_command::<MockModule>(MockCommand::Valid, None)
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;

        // alice must have received something
        assert!(!alice.received_nothing());
    }
}
