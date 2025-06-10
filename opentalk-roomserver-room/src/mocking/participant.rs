// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use axum::extract::ws::Message;
use opentalk_roomserver_signaling::{
    signaling_event::SignalingEvent, signaling_module::SignalingModule,
};
use opentalk_roomserver_types::{
    client_parameters::{ClientKind, ClientParameters, Role},
    core_event::CoreEvent,
    join::join_success::JoinSuccess,
    signaling::SignalingCommand,
};
use opentalk_roomserver_web_api::v1::signaling::websocket;
use opentalk_types_api_v1::users::PublicUserProfile;
use opentalk_types_common::users::{DisplayName, UserId, UserInfo};
use opentalk_types_signaling::ParticipantId;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{json, value::to_raw_value};
use tokio::sync::mpsc;

use super::{
    room::{self, TestRoom},
    socket::MockSocket,
};

#[derive(Debug)]
pub enum ReceiveError {
    Closed,
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
}

impl<S> MockParticipant<S> {
    pub fn alice() -> MockParticipantBuilder<PublicUserProfile> {
        let profile = PublicUserProfile {
            id: UserId::from_u128(0x1),
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
            secret: "Alice Device A".to_string(),
        }
    }

    pub fn bob() -> MockParticipantBuilder<PublicUserProfile> {
        let profile = PublicUserProfile {
            id: UserId::from_u128(0x2),
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
            secret: "Bob Device A".to_string(),
        }
    }

    pub fn charlie() -> MockParticipantBuilder<PublicUserProfile> {
        let profile = PublicUserProfile {
            id: UserId::from_u128(0x3),
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
            secret: "Charlie Device A".to_string(),
        }
    }

    pub fn gustav() -> MockParticipantBuilder<DisplayName> {
        MockParticipantBuilder {
            profile: "Gustav the great".parse().expect("Valid DisplayName"),
            role: Role::User,
            secret: "Gustav Device A".to_string(),
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

    pub async fn receive_event<M>(&mut self) -> Result<SignalingEvent<M::Outgoing>, ReceiveError>
    where
        M: SignalingModule,
        M::Outgoing: DeserializeOwned,
    {
        let Some(received) = self.receiver.recv().await else {
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
        let Some(received) = self.receiver.recv().await else {
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
    secret: String,
}

impl<P> MockParticipantBuilder<P> {
    pub fn secret(mut self, secret: String) -> Self {
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
