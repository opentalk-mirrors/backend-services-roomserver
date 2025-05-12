// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeMap;

use opentalk_roomserver_signaling::signaling_module::{FatalError, SharedRawJson};
use opentalk_roomserver_types::{
    client_parameters::{ClientKind, ClientParameters},
    connection_id::ConnectionId,
    error::SignalingError,
};
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;
use opentalk_types_common::{
    events::{EventInfo, MeetingDetails},
    modules::{ModuleId, module_id},
};
use opentalk_types_signaling::{ModuleData, ModulePeerData, Participant, ParticipantId, Role};
use opentalk_types_signaling_control::{event::JoinSuccess, room::RoomInfo};
use serde::{Deserialize, Serialize};

use super::RoomTask;
use crate::{
    message_router::CloseReason,
    signaling::{DynBroadcastEvent, dyn_module_context::DynModuleContext},
};

pub const NAMESPACE: ModuleId = module_id!("core");

/// Outgoing websocket messages in the core namespace
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreEvent {
    /// Message sent to a participant on a successful join
    JoinSuccess(Box<JoinSuccess>),

    /// Broadcast message sent to all participants when a new participant has joined
    ParticipantConnected {
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        peer_join_info: BTreeMap<ModuleId, SharedRawJson>, // TODO: find a better name
    },

    /// Broadcast message sent to all participants when a participant disconnected
    ParticipantDisconnected {
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        reason: DisconnectReason,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisconnectReason {
    /// The participant left the conference
    Leave,
    /// The connection was interrupted
    ConnectionLost,
    /// The participant was removed due to an internal error
    InternalError,
}

impl From<CloseReason> for DisconnectReason {
    fn from(value: CloseReason) -> Self {
        match value {
            CloseReason::ParticipantClosed => DisconnectReason::Leave,
            CloseReason::ConnectionLost => DisconnectReason::ConnectionLost,
            CloseReason::InternalError | CloseReason::TaskClosed => DisconnectReason::InternalError,
        }
    }
}

impl<Socket: SignalingSocket> RoomTask<Socket> {
    /// A participant connected to the conference
    ///
    /// Sends the [`CoreEvent::JoinSuccess`] to the joining participant and notifies other participants with the
    /// [`CoreEvent::ParticipantConnected`] message.
    pub(super) async fn participant_joined(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        client_parameters: ClientParameters,
    ) -> Result<(), FatalError> {
        let mut module_data = ModuleData::new();

        let mut peer_module_data = BTreeMap::new();

        self.broadcast_event_to_modules(
            participant_id,
            connection_id,
            DynBroadcastEvent::Connected {
                participant_id,
                connection_id,
                module_data: &mut module_data,
                peer_module_data: &mut peer_module_data,
            },
        )
        .await;

        let join_success_msg = build_join_success(
            &self.context(participant_id, connection_id),
            participant_id,
            client_parameters,
            module_data,
        );
        self.serialize_and_send(
            [connection_id],
            NAMESPACE,
            CoreEvent::JoinSuccess(Box::new(join_success_msg)),
        )
        .await?;

        for (&peer_id, state) in self.participants.connected() {
            let peer_join_info = peer_module_data.remove(&peer_id);

            let connections = state
                .connections
                .keys()
                .copied()
                .filter(|&c| c != connection_id);

            self.serialize_and_send(
                connections,
                NAMESPACE,
                CoreEvent::ParticipantConnected {
                    participant_id,
                    connection_id,
                    peer_join_info: peer_join_info.unwrap_or_default(),
                },
            )
            .await?;
        }

        Ok(())
    }

    /// Inform modules that the participant has left the conference and broadcast [`CoreEvent::ParticipantDisconnected`]
    /// to all participants
    pub(super) async fn participant_disconnected(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        reason: DisconnectReason,
    ) {
        self.broadcast_event_to_modules(
            participant_id,
            connection_id,
            DynBroadcastEvent::Disconnected {
                participant_id,
                connection_id,
            },
        )
        .await;

        let content = CoreEvent::ParticipantDisconnected {
            participant_id,
            connection_id,
            reason,
        };

        self.serialize_and_broadcast(NAMESPACE, content)
            .await
            .expect("CoreEvent::ParticipantDisconnected must be serializable");
    }

    /// Broadcast the [`DynBroadcastEvent`] to all modules
    pub(crate) async fn broadcast_event_to_modules(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        mut event: DynBroadcastEvent<'_>,
    ) {
        let mut errors = Vec::new();
        for (namespace, module) in self.modules.iter_mut() {
            if let Err(err) = module
                .on_broadcast_event(
                    &mut DynModuleContext::new(
                        self.info.room_id,
                        participant_id,
                        connection_id,
                        &mut self.info,
                        &mut self.message_router,
                        &mut self.participants,
                        &self.loopback_futures,
                    ),
                    &mut event,
                )
                .await
            {
                errors.push((namespace.clone(), err));
            }
        }

        for (namespace, err) in errors {
            self.handle_fatal_module_error(namespace, err).await;
        }
    }

    /// An unrecoverable module error occurred and the module needs to be removed for the remainder of the conference
    ///
    /// Further requests to the module will result in a [`SignalingError::UnknownNamespace`] error.
    pub(crate) async fn handle_fatal_module_error(&mut self, namespace: ModuleId, err: FatalError) {
        log::error!(
            "The {namespace} module caused a fatal error and will be shut down: {:#?}",
            err.0
        );

        let Some(module) = self.modules.remove(&namespace) else {
            log::error!("Attempted to remove non-existent module {namespace}");
            return;
        };

        module.destroy().await;

        // Remove the module from the room state
        self.info.room.tariff.modules.remove(&namespace);

        self.broadcast_error(SignalingError::FatalModuleError { namespace })
            .await;
    }
}

fn build_join_success(
    ctx: &DynModuleContext<'_>,
    participant_id: ParticipantId,
    client_parameters: ClientParameters,
    module_data: ModuleData,
) -> JoinSuccess {
    let participants = ctx
        .participants
        .all
        .iter()
        .map(|_id| Participant {
            id: participant_id,
            module_data: ModulePeerData::new(), // TODO: needs implementation in the signaling module
        })
        .collect();

    let (display_name, role, avatar_url, is_room_owner) = match client_parameters.kind {
        ClientKind::Registered { profile } => (
            profile.user_info.display_name,
            Role::User,
            Some(profile.user_info.avatar_url),
            ctx.room_info.room.created_by.id == profile.id,
        ),
        ClientKind::Guest { display_name } => (display_name, Role::Guest, None, false),
    };

    let event_info = ctx.room_info.room.event.as_ref().map(|event_context| {
        let meeting_details = MeetingDetails {
            invite_code_id: ctx.room_info.room.invite_code,
            call_in: ctx.room_info.room.call_in.clone(),
            streaming_links: ctx.room_info.room.streaming_links.clone(),
        };
        EventInfo {
            id: event_context.id,
            room_id: ctx.room_id,
            title: event_context.title.clone(),
            is_adhoc: event_context.is_adhoc,
            meeting_details: Some(meeting_details),
            e2e_encryption: ctx.room_info.room.e2e_encryption,
        }
    });

    JoinSuccess {
        id: participant_id,
        display_name,
        avatar_url,
        role,
        closes_at: ctx.room_info.closes_at,
        tariff: Box::new(ctx.room_info.room.tariff.clone()),
        participants,
        event_info,
        room_info: RoomInfo {
            id: ctx.room_id,
            password: ctx.room_info.room.password.clone(),
            created_by: ctx.room_info.room.created_by.user_info.clone(),
        },
        is_room_owner,
        module_data,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use opentalk_roomserver_signaling::signaling_module::SharedRawJson;
    use opentalk_roomserver_types::connection_id::ConnectionId;
    use opentalk_types_common::{
        modules::module_id,
        rooms::RoomId,
        tariffs::TariffResource,
        users::{DisplayName, UserInfo},
        utils::ExampleData,
    };
    use opentalk_types_signaling::{ModuleData, ParticipantId, Role};
    use opentalk_types_signaling_control::{event::JoinSuccess, room::RoomInfo};
    use pretty_assertions::assert_eq;
    use serde_json::{json, value::to_raw_value};

    use super::{CoreEvent, DisconnectReason};

    #[test]
    fn serialize_core_event_success() {
        let join_success = JoinSuccess {
            id: ParticipantId::nil(),
            display_name: DisplayName::example_data(),
            avatar_url: None,
            role: Role::Guest,
            closes_at: None,
            tariff: Box::new(TariffResource::example_data()),
            module_data: ModuleData::new(),
            participants: vec![],
            event_info: None,
            room_info: RoomInfo {
                id: RoomId::nil(),
                password: None,
                created_by: UserInfo::example_data(),
            },
            is_room_owner: false,
        };
        let event = CoreEvent::JoinSuccess(Box::new(join_success));
        let json = serde_json::to_value(&event).unwrap();

        assert_eq!(
            json,
            json!({
                "join_success": {
                    "id": "00000000-0000-0000-0000-000000000000",
                    "display_name": "Alice Adams",
                    "role": "guest",
                    "tariff": {
                        "id": "00000000-0000-0000-0000-000000000000",
                        "name": "Starter tariff",
                        "quotas": {
                            "max_storage": 50000
                        },
                        "modules": {
                            "chat": {
                                "features": []
                            },
                            "core": {
                                "features": []
                            },
                            "livekit": {
                                "features": []
                            },
                            "moderation": {
                                "features": []
                            },
                            "recording": {
                                "features": [
                                    "record"
                                ]
                            }
                        }
                    },
                    "participants": [],
                    "event_info": null,
                    "room_info": {
                        "id": "00000000-0000-0000-0000-000000000000",
                        "created_by": {
                            "title": "",
                            "firstname": "Alice",
                            "lastname": "Adams",
                            "display_name": "Alice Adams",
                            "avatar_url": "https://gravatar.com/avatar/c160f8cc69a4f0bf2b0362752353d060"
                        }
                    },
                    "is_room_owner": false
                }
            })
        );
    }

    #[test]
    fn serialize_core_event_joined() {
        let mut peer_join_info = BTreeMap::new();
        peer_join_info.insert(
            module_id!("test"),
            SharedRawJson::from(
                to_raw_value(&json!({
                    "key": "value"
                }))
                .unwrap(),
            ),
        );

        let event = CoreEvent::ParticipantConnected {
            participant_id: ParticipantId::nil(),
            connection_id: ConnectionId::nil(),
            peer_join_info,
        };
        let json = serde_json::to_value(&event).unwrap();

        assert_eq!(
            json,
            json!({
              "participant_connected": {
                "participant_id": "00000000-0000-0000-0000-000000000000",
                "connection_id": "00000000-0000-0000-0000-000000000000",
                "peer_join_info": {
                  "test": {
                    "key": "value"
                  }
                }
              }
            })
        );
    }

    #[test]
    fn serialize_core_event_disconnected() {
        let event = CoreEvent::ParticipantDisconnected {
            participant_id: ParticipantId::nil(),
            connection_id: ConnectionId::nil(),
            reason: DisconnectReason::ConnectionLost,
        };

        let json = serde_json::to_value(&event).unwrap();

        assert_eq!(
            json,
            json!({
              "participant_disconnected": {
                "participant_id": "00000000-0000-0000-0000-000000000000",
                "connection_id": "00000000-0000-0000-0000-000000000000",
                "reason": "connection_lost"
              }
            })
        );
    }
}
