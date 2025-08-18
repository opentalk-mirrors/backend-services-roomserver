// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{cell::RefCell, collections::BTreeMap, sync::Arc};

use anyhow::anyhow;
use opentalk_roomserver_signaling::event_origin::{EventOrigin, ParticipantOrigin};
use opentalk_roomserver_types::{
    breakout::module_data::BreakoutPeerModuleData,
    client_parameters::{ClientKind, Role as RoomserverClientRole},
    connection_id::ConnectionId,
    core::{CORE_MODULE_ID, CoreEvent},
    device_id::DeviceId,
    disconnect_reason::DisconnectReason,
    error::SignalingError,
    join::{
        connection_info::ConnectionInfo, event_info::EventInfo, join_success::JoinSuccess,
        participant::Participant,
    },
    room_info::RoomInfo,
    room_kind::RoomKind,
    signaling::module_error::FatalError,
};
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;
use opentalk_types_common::{events::MeetingDetails, modules::ModuleId, time::Timestamp};
use opentalk_types_signaling::{ModuleData, ModulePeerData, ParticipantId, Role};

use super::RoomTask;
use crate::signaling::{DynBroadcastEvent, dyn_module_context::DynModuleContext};

impl<Socket: SignalingSocket> RoomTask<Socket> {
    /// A participant connected to the conference
    ///
    /// Sends the [`CoreEvent::JoinSuccess`] to the joining participant and notifies other participants with the
    /// [`CoreEvent::ParticipantConnected`] message.
    pub(super) async fn participant_joined(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        device_id: DeviceId,
        client_kind: ClientKind,
        role: RoomserverClientRole,
    ) -> Result<(), FatalError> {
        let mut module_data = ModuleData::new();

        let mut peer_module_data = BTreeMap::new();

        let Some(current_breakout_room) = self
            .participants
            .all_unfiltered
            .get(&participant_id)
            .map(|s| s.room)
        else {
            return Err(FatalError(anyhow!(
                "Unable to get participant state for fresh connections"
            )));
        };

        let participant_origin = ParticipantOrigin {
            id: participant_id,
            connection_id,
            transaction_id: None,
        };

        self.broadcast_event_to_modules(
            participant_origin.into(),
            current_breakout_room,
            DynBroadcastEvent::Connected {
                participant_id,
                connection_id,
                module_data: &mut module_data,
                peer_module_data: &mut peer_module_data,
            },
        )
        .await;

        self.add_breakout_module_data(&mut module_data, current_breakout_room);

        let join_success_msg = self.build_join_success(
            participant_id,
            connection_id,
            device_id,
            client_kind,
            role,
            module_data,
        );

        self.message_router
            .conference
            .serialize_and_send(
                [connection_id],
                CORE_MODULE_ID,
                None,
                CoreEvent::JoinSuccess(Box::new(join_success_msg)),
            )
            .await?;

        for (&peer_id, state) in self.participants.connected().iter() {
            let peer_join_info = peer_module_data.remove(&peer_id);

            let connections = state
                .connections
                .keys()
                .copied()
                .filter(|&c| c != connection_id);

            self.message_router
                .conference
                .serialize_and_send(
                    connections,
                    CORE_MODULE_ID,
                    None,
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
        origin: EventOrigin,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        room: RoomKind,
        reason: DisconnectReason,
    ) {
        self.broadcast_event_to_modules(
            origin,
            room,
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

        self.message_router
            .conference
            .serialize_and_broadcast(CORE_MODULE_ID, None, content)
            .await
            .expect("CoreEvent::ParticipantDisconnected must be serializable");
    }

    /// Broadcast the [`DynBroadcastEvent`] to all modules
    pub(crate) async fn broadcast_event_to_modules(
        &mut self,
        origin: EventOrigin,
        room_scope: RoomKind,
        mut event: DynBroadcastEvent<'_>,
    ) {
        let mut errors = Vec::new();
        let mut messages = RefCell::new(Vec::new());
        let timestamp = Timestamp::now();
        for (namespace, module) in self.modules.iter_mut() {
            if let Err(err) = module
                .on_broadcast_event(
                    &mut DynModuleContext::new(
                        self.info.room_id,
                        room_scope,
                        origin,
                        &mut self.info,
                        &mut self.participants,
                        &mut self.waiting_participants,
                        timestamp,
                        Arc::clone(&self.storage),
                        &mut messages,
                        &mut self.loopback_futures,
                    ),
                    &mut event,
                )
                .await
            {
                errors.push((namespace.clone(), err));
            }
        }

        self.handle_module_messages(messages, room_scope, origin, timestamp)
            .await;

        for (namespace, err) in errors {
            self.handle_fatal_module_error(namespace, origin.transaction_id(), err)
                .await;
        }
    }

    /// An unrecoverable module error occurred and the module needs to be removed for the remainder of the conference
    ///
    /// Further requests to the module will result in a [`SignalingError::UnknownNamespace`] error.
    pub(crate) async fn handle_fatal_module_error(
        &mut self,
        namespace: ModuleId,
        transaction_id: Option<u64>,
        err: FatalError,
    ) {
        tracing::error!(
            "The {namespace} module caused a fatal error and will be shut down: {:#?}",
            err.0
        );

        let Some(module) = self.modules.remove(&namespace) else {
            tracing::error!("Attempted to remove non-existent module {namespace}");
            return;
        };

        module.destroy(self.info.room_id).await;

        // Remove the module from the room state
        self.info.room.tariff.modules.remove(&namespace);

        self.message_router
            .conference
            .broadcast_error(
                transaction_id,
                SignalingError::FatalModuleError { namespace },
            )
            .await;
    }

    fn build_join_success(
        &self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        device_id: DeviceId,
        client_kind: ClientKind,
        role: RoomserverClientRole,
        module_data: ModuleData,
    ) -> JoinSuccess {
        let participants = self
            .participants
            .all_unfiltered
            .iter()
            .filter(|(id, ..)| id != &&participant_id)
            .map(|(id, state)| {
                let connections = state
                    .connections
                    .iter()
                    .map(|(conn, device)| ConnectionInfo {
                        connection_id: *conn,
                        device_id: *device,
                    })
                    .collect();

                let mut module_peer_data = ModulePeerData::new();

                // TODO: temporary solution to let participants know which participant is in which breakout room
                module_peer_data
                    .insert(&BreakoutPeerModuleData { room: state.room })
                    .expect("BreakoutPeerModuleData must be serializable");

                Participant {
                    id: *id,
                    connections,
                    module_data: module_peer_data, // TODO: needs implementation in the signaling module
                }
            })
            .collect();

        let display_name = client_kind.display_name();
        let (role, avatar_url, is_room_owner) = match client_kind {
            ClientKind::Registered { profile } => (
                role.to_opentalk_types_signaling_role(),
                Some(profile.user_info.avatar_url),
                self.info.room.created_by.id == profile.id,
            ),
            ClientKind::Guest { .. } => (Role::Guest, None, false),
            ClientKind::Recorder => (Role::Guest, None, false),
        };

        let event_info = self
            .info
            .room
            .event
            .as_ref()
            .map(|event_context| EventInfo {
                id: event_context.id,
                room_id: self.info.room_id,
                title: event_context.title.clone(),
                is_adhoc: event_context.is_adhoc,
                e2e_encryption: self.info.room.e2e_encryption,
            });

        let meeting_details = MeetingDetails {
            invite_code_id: self.info.room.invite_code,
            call_in: self.info.room.call_in.clone(),
            streaming_links: self.info.room.streaming_links.clone(),
        };

        let other_connections = self
            .participants
            .all_unfiltered
            .get(&participant_id)
            .map(|state| {
                state
                    .connections
                    .iter()
                    .filter(|(conn, ..)| conn != &&connection_id)
                    .map(|(conn, device)| ConnectionInfo {
                        connection_id: *conn,
                        device_id: *device,
                    })
                    .collect()
            })
            .unwrap_or_default();

        JoinSuccess {
            id: participant_id,
            connection_id,
            device_id,
            connections: other_connections,
            display_name,
            avatar_url,
            role,
            closes_at: self.info.closes_at,
            tariff: Box::new(self.info.room.tariff.clone()),
            participants,
            event_info,
            room_info: RoomInfo {
                id: self.info.room_id,
                password: self.info.room.password.clone(),
                created_by: self.info.room.created_by.user_info.clone(),
            },
            meeting_details,
            is_room_owner,
            module_data,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use opentalk_roomserver_signaling::signaling_event::SignalingEvent;
    use opentalk_roomserver_types::{
        connection_id::ConnectionId,
        core::CORE_MODULE_ID,
        device_id::DeviceId,
        join::{connection_info::ConnectionInfo, join_success::JoinSuccess},
        room_info::RoomInfo,
        shared_raw_json::SharedRawJson,
    };
    use opentalk_types_common::{
        events::MeetingDetails,
        modules::{ModuleId, module_id},
        rooms::RoomId,
        tariffs::TariffResource,
        users::{DisplayName, UserInfo},
        utils::ExampleData,
    };
    use opentalk_types_signaling::{ModuleData, ParticipantId, Role, SignalingModuleFrontendData};
    use pretty_assertions::assert_eq;
    use serde::{Deserialize, Serialize};
    use serde_json::{json, value::to_raw_value};

    use super::{CoreEvent, DisconnectReason};

    fn test_module_data() -> ModuleData {
        #[derive(Debug, Serialize, Deserialize)]
        struct TestData {
            a: String,
            b: u64,
        }

        impl SignalingModuleFrontendData for TestData {
            const NAMESPACE: Option<ModuleId> = Some(module_id!("test"));
        }

        let mut module_data = ModuleData::new();

        module_data
            .insert(&TestData {
                a: "test".into(),
                b: 42,
            })
            .unwrap();

        module_data
    }

    #[test]
    fn serialize_signaling_event_success() {
        let join_success = JoinSuccess {
            id: ParticipantId::nil(),
            connection_id: ConnectionId::nil(),
            device_id: DeviceId::nil(),
            connections: vec![ConnectionInfo {
                connection_id: ConnectionId::nil(),
                device_id: DeviceId::nil(),
            }],
            display_name: DisplayName::example_data(),
            avatar_url: None,
            role: Role::Guest,
            closes_at: None,
            tariff: Box::new(TariffResource::example_data()),
            module_data: test_module_data(),
            participants: vec![],
            event_info: None,
            room_info: RoomInfo {
                id: RoomId::nil(),
                password: None,
                created_by: UserInfo::example_data(),
            },
            meeting_details: MeetingDetails {
                invite_code_id: None,
                call_in: None,
                streaming_links: vec![],
            },
            is_room_owner: false,
        };
        let event = SignalingEvent {
            namespace: CORE_MODULE_ID,
            payload: CoreEvent::JoinSuccess(join_success.into()),
            transaction_id: Some(0),
        };
        let json = serde_json::to_value(&event).unwrap();

        assert_eq!(
            json,
            json!({
                "namespace": "core",
                "transaction_id": 0,
                "payload": {
                    "join_success": {
                        "id": "00000000-0000-0000-0000-000000000000",
                        "connection_id": "00000000-0000-0000-0000-000000000000",
                        "device_id": "00000000-0000-0000-0000-000000000000",
                        "connections": [
                            {
                                "connection_id": "00000000-0000-0000-0000-000000000000",
                                "device_id": "00000000-0000-0000-0000-000000000000"
                            }
                        ],
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
                        "module_data": {
                            "test": {
                                "a": "test",
                                "b": 42
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
                        "meeting_details": {
                            "streaming_links": []
                        },

                        "is_room_owner": false
                    }
                }
            })
        );

        let raw = r#"{
    "namespace": "core",
    "payload": {
        "join_success": {
            "id": "00000000-0000-0000-0000-000000000001",
            "connection_id": "00000000-0000-0000-0000-000000000001",
            "device_id": "00000000-0000-0000-0000-000000000001",
            "connections": [
                {
                    "connection_id": "00000000-0000-0000-0000-000000000000",
                    "device_id": "00000000-0000-0000-0000-000000000000"
                }
            ],
            "display_name": "Alice the angry",
            "avatar_url": "https://example.com/avatar-of-alice",
            "role": "user",
            "tariff": {
                "id": "00000000-0000-0000-0000-000000000000",
                "name": "Starter tariff",
                "quotas": {
                    "max_storage": 50000
                },
                "modules": {
                    "chat": {
                        "features": []
                    }
                }
            },
            "chat": {
                "enabled": true,
                "room_history": [],
                "groups_history": [],
                "private_history": [],
                "last_seen_timestamp_global": null,
                "last_seen_timestamps_private": {},
                "last_seen_timestamps_group": {}
            },
            "participants": [],
            "module_data": {},
            "event_info": {
                "id": "00000000-0000-0000-0000-004433221100",
                "room_id": "00000000-0000-0000-0000-000000000001",
                "title": "Team Event",
                "is_adhoc": false,
                "e2e_encryption": false
            },
            "room_info": {
                "id": "00000000-0000-0000-0000-000000000001",
                "password": "1234",
                "created_by": {
                    "title": "",
                    "firstname": "Alice",
                    "lastname": "Adams",
                    "display_name": "Alice Adams",
                    "avatar_url": "https://gravatar.com/avatar/c160f8cc69a4f0bf2b0362752353d060"
                }
            },
            "meeting_details": {
                "invite_code_id": "00000000-0000-0000-0000-0000deadbeef",
                "call_in": {
                    "tel": "+555-123-456-789",
                    "id": "1234567890",
                    "password": "0987654321"
                },
                "streaming_links": [
                    {
                        "name": "My OwnCast Stream",
                        "url": "https://owncast.example.com/mystream"
                    }
                ]
            },
            "is_room_owner": false
        }
    }
}"#;
        let _: SignalingEvent<CoreEvent> = serde_json::from_str(raw).unwrap();
    }

    #[test]
    fn serialize_core_event_success() {
        let join_success = JoinSuccess {
            id: ParticipantId::nil(),
            connection_id: ConnectionId::nil(),
            device_id: DeviceId::nil(),
            connections: vec![],
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
            meeting_details: MeetingDetails {
                invite_code_id: None,
                call_in: None,
                streaming_links: vec![],
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
                    "connection_id": "00000000-0000-0000-0000-000000000000",
                    "device_id": "00000000-0000-0000-0000-000000000000",
                    "connections": [],
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
                    "module_data": {},
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
                    "meeting_details": {
                        "streaming_links": [],
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
