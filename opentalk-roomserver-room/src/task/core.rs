// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    cell::RefCell,
    collections::{BTreeMap, hash_map::Entry},
    sync::Arc,
};

use anyhow::{Context, anyhow};
use opentalk_roomserver_signaling::{
    event_origin::{EventOrigin, ParticipantOrigin},
    module_context::ModuleMessage,
    participant_state::ParticipantState,
    signaling_event::SignalingEvent,
};
use opentalk_roomserver_types::{
    client_parameters::{ClientKind, Role as RoomserverClientRole},
    connection_id::ConnectionId,
    core::{CORE_MODULE_ID, CoreCommand, CoreError, CoreEvent, LeftWaitingRoom, state::CoreState},
    device_id::DeviceId,
    disconnect_reason::DisconnectReason,
    error::SignalingError,
    join::{
        connection_info::ConnectionInfo, event_info::EventInfo, join_success::JoinSuccess,
        participant::Participant,
    },
    room_info::RoomInfo,
    room_kind::RoomKind,
    shared_json::SharedJson,
    shared_raw_json::SharedRawJson,
    signaling::{
        SignalingCommand,
        module_error::{FatalError, SignalingModuleError},
    },
};
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;
use opentalk_types_common::{
    events::MeetingDetails, modules::ModuleId, tariffs::QuotaType, time::Timestamp,
};
use opentalk_types_signaling::{ModuleData, ParticipantId, Role};

use super::RoomTask;
use crate::{
    signaling::{DynBroadcastEvent, dyn_module_context::DynModuleContext},
    task::participant_id_from_uuid,
};

impl<Socket: SignalingSocket> RoomTask<Socket> {
    pub(crate) fn handle_core_command(
        &mut self,
        participant_origin: ParticipantOrigin,
        command: SignalingCommand,
    ) {
        let core_command: CoreCommand = match serde_json::from_str(command.payload.get()) {
            Ok(command) => command,
            Err(err) => {
                tracing::warn!("🚨🚨🚨 received unsupported core command 🚨🚨🚨");
                self.message_router_for_participant(participant_origin.id)
                    .send_error(
                        participant_origin.connection_id,
                        participant_origin.transaction_id,
                        SignalingError::InvalidJson {
                            message: format!("{err:?}"),
                        },
                    );
                return;
            }
        };

        let result = match core_command {
            CoreCommand::EnterRoom => self.enter_room(participant_origin),
        };

        let router = self.message_router_for_participant(participant_origin.id);
        if let Err(e) = result {
            match e {
                SignalingModuleError::Internal(err) => {
                    tracing::error!("internal error in core module: {err:?}");

                    router.send_error(
                        participant_origin.connection_id,
                        command.transaction_id,
                        SignalingError::Internal,
                    );
                }
                SignalingModuleError::Fatal(err) => {
                    tracing::error!("fatal error in core module: {err:?}");

                    router.send_error(
                        participant_origin.connection_id,
                        command.transaction_id,
                        SignalingError::Internal,
                    );
                }
                SignalingModuleError::Module(module_error) => {
                    let result = router.serialize_and_send(
                        [participant_origin.connection_id],
                        CORE_MODULE_ID,
                        command.transaction_id,
                        CoreEvent::Error(module_error),
                    );

                    if let Err(fatal_error) = result {
                        tracing::error!("failed to send error in core module: {fatal_error:?}");

                        router.send_error(
                            participant_origin.connection_id,
                            command.transaction_id,
                            SignalingError::Internal,
                        );
                    }
                }
                SignalingModuleError::NotSupported => {
                    router.send_error(
                        participant_origin.connection_id,
                        command.transaction_id,
                        SignalingError::NotSupported,
                    );
                }
            }
        }
    }

    fn enter_room(
        &mut self,
        participant_origin: ParticipantOrigin,
    ) -> Result<(), SignalingModuleError<CoreError>> {
        let Entry::Occupied(participant) = self.waiting_participants.entry(participant_origin.id)
        else {
            tracing::debug!("Failed to enter room: participant not known");
            return Err(CoreError::UnknownParticipant.into());
        };

        if !participant.get().accepted {
            tracing::debug!("Failed to enter room: participant not yet accepted");
            return Err(CoreError::NotAccepted.into());
        }
        let participant = participant.remove();

        let moderator_ids = self.participants.connected().moderators().connection_ids();

        self.message_router.conference.serialize_and_send(
            moderator_ids,
            CORE_MODULE_ID,
            None,
            CoreEvent::LeftWaitingRoom(LeftWaitingRoom {
                id: participant_origin.id,
                connection_id: participant_origin.connection_id,
            }),
        )?;

        self.message_router
            .upgrade_connections(participant.connections.keys());

        for (&connection_id, &device_id) in &participant.connections {
            self.join_room(
                participant_origin.id,
                connection_id,
                device_id,
                participant.kind.clone(),
                participant.role,
            )?;
        }

        Ok(())
    }

    /// A participant connected to the conference
    ///
    /// Sends the [`CoreEvent::JoinSuccess`] to the connection of the participant that joins.
    /// Notifies other connections with the [`CoreEvent::ParticipantConnected`] message.
    ///
    /// NOTE: In case the joining participant is already connected with another device, they will
    /// also receive the [`CoreEvent::ParticipantConnected`] messages on the device that is already
    /// connected.
    pub(super) fn participant_joined(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        device_id: DeviceId,
        client_kind: ClientKind,
        role: RoomserverClientRole,
    ) -> Result<(), FatalError> {
        let mut own_data = ModuleData::new();
        let mut peer_events = BTreeMap::new();
        let mut peer_data = BTreeMap::new();

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

        let actions = self.broadcast_event_to_modules(
            participant_origin.into(),
            current_breakout_room,
            DynBroadcastEvent::Connected {
                participant_id,
                connection_id,
                own_data: &mut own_data,
                peer_events: &mut peer_events,
                peer_data: &mut peer_data,
            },
        );

        self.join_success_breakout_own_data(&mut own_data, current_breakout_room);
        self.join_success_core_peer_events(participant_id, &mut peer_events)?;

        let join_success_msg = self.build_join_success(
            participant_id,
            connection_id,
            device_id,
            client_kind,
            role,
            own_data,
            peer_data.clone(),
        )?;

        self.message_router.conference.serialize_and_send(
            [connection_id],
            CORE_MODULE_ID,
            None,
            CoreEvent::JoinSuccess(Box::new(join_success_msg)),
        )?;

        for (&peer_id, state) in self.participants.connected().iter() {
            let peer_join_info = peer_events.remove(&peer_id);

            let connections = state
                .connections
                .keys()
                .copied()
                .filter(|&c| c != connection_id);

            self.message_router.conference.serialize_and_send(
                connections,
                CORE_MODULE_ID,
                None,
                CoreEvent::ParticipantConnected {
                    participant_id,
                    connection_id,
                    peer_data: peer_join_info.unwrap_or_default(),
                },
            )?;
        }

        actions.handle_requested_messages(self)?;

        // Start the quota timer if the room has a time limit
        if self
            .info
            .room
            .tariff
            .quotas
            .contains_key(&QuotaType::RoomTimeLimitSecs)
        {
            // This will have no effect if the timer is already running
            self.quota_timeout.start();
        }

        Ok(())
    }

    /// Inform modules that the participant has left the conference and broadcast
    /// [`CoreEvent::ParticipantDisconnected`] to all participants
    pub(super) fn participant_disconnected(
        &mut self,
        origin: EventOrigin,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        room: RoomKind,
        reason: DisconnectReason,
    ) -> Result<(), FatalError> {
        self.broadcast_event_to_modules(
            origin,
            room,
            DynBroadcastEvent::Disconnected {
                participant_id,
                connection_id,
            },
        )
        .handle_requested_messages(self)?;

        let content = CoreEvent::ParticipantDisconnected {
            participant_id,
            connection_id,
            reason,
        };

        self.message_router
            .conference
            .serialize_and_broadcast(CORE_MODULE_ID, None, content)?;

        Ok(())
    }

    #[tracing::instrument(skip(self, event), fields(%event))]
    pub(crate) fn broadcast_event_to_modules(
        &mut self,
        origin: EventOrigin,
        room_kind: RoomKind,
        mut event: DynBroadcastEvent<'_>,
    ) -> RequestedModuleActions {
        let mut errors = Vec::new();
        let mut messages = RefCell::new(Vec::new());
        let timestamp = Timestamp::now();
        for (namespace, module) in &mut self.modules {
            if let Err(err) = module.on_broadcast_event(
                &mut DynModuleContext::new(
                    self.info.room_id,
                    room_kind,
                    origin,
                    &mut self.info,
                    &mut self.participants,
                    &mut self.waiting_participants,
                    &mut self.banned_participants,
                    timestamp,
                    Arc::clone(&self.storage),
                    &mut messages,
                    &mut self.loopback_futures,
                ),
                &mut event,
            ) {
                errors.push((namespace.clone(), err));
            }
        }

        RequestedModuleActions {
            messages,
            errors,
            timestamp,
            room_kind,
            origin,
        }
    }

    /// An unrecoverable module error occurred and the module needs to be removed for the remainder
    /// of the conference
    ///
    /// Further requests to the module will result in a [`SignalingError::UnknownNamespace`] error.
    pub(crate) fn handle_fatal_module_error(
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

        module.destroy(self.info.room_id, Arc::clone(&self.storage));

        // Remove the module from the room state
        self.info.room.tariff.modules.remove(&namespace);

        self.message_router.conference.broadcast_error(
            transaction_id,
            SignalingError::FatalModuleError { namespace },
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn build_join_success(
        &self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        device_id: DeviceId,
        client_kind: ClientKind,
        role: RoomserverClientRole,
        module_data: ModuleData,
        mut participants_module_data: BTreeMap<ParticipantId, BTreeMap<ModuleId, SharedJson>>,
    ) -> Result<JoinSuccess, FatalError> {
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

                let mut module_peer_data = participants_module_data.remove(id).unwrap_or_default();
                Self::join_success_breakout_peer_data(&mut module_peer_data, state)?;
                self.join_success_core_peer_data(participant_id, state, &mut module_peer_data)?;

                Ok(Participant {
                    id: *id,
                    connections,
                    module_data: module_peer_data,
                })
            })
            .collect::<Result<Vec<_>, FatalError>>()?;

        let display_name = client_kind.display_name();
        let (role, avatar_url, is_room_owner) = match client_kind {
            ClientKind::Registered { profile } | ClientKind::RegisteredCallIn { profile } => (
                role.to_opentalk_types_signaling_role(),
                Some(profile.user_info.avatar_url),
                self.info.room.created_by.id == profile.id,
            ),
            ClientKind::Guest { .. } | ClientKind::Recorder | ClientKind::CallIn { .. } => {
                (Role::Guest, None, false)
            }
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

        Ok(JoinSuccess {
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
        })
    }

    /// Sends a [`CoreEvent::TimeLimitQuotaElapsed`] to all participants in the conference and
    /// waiting room
    pub(crate) fn time_limit_quota_elapsed(&mut self) {
        let event = SignalingEvent {
            namespace: CORE_MODULE_ID,
            transaction_id: None,
            timestamp: Timestamp::now(),
            payload: CoreEvent::TimeLimitQuotaElapsed,
        };
        let Ok(raw_value) = serde_json::value::to_raw_value(&event) else {
            tracing::error!("Failed to serialize CoreEvent");
            return;
        };

        let shared_json: SharedRawJson = raw_value.into();
        self.message_router
            .conference
            .broadcast_event(shared_json.clone(), &[]);

        self.message_router
            .waiting_room
            .broadcast_event(shared_json, &[]);
    }

    /// Attach core related info about the joining participant for other participants (peers).
    ///
    /// This will be sent to all other participants, but not the joining participant.
    pub(crate) fn join_success_core_peer_events(
        &self,
        participant_id: ParticipantId,
        peer_events: &mut BTreeMap<ParticipantId, BTreeMap<ModuleId, SharedJson>>,
    ) -> Result<(), FatalError> {
        let Some(joinee) = self.participants.all_unfiltered.get(&participant_id) else {
            return Err(FatalError(anyhow::anyhow!("joining participant not found")));
        };
        let data = CoreState {
            display_name: joinee.kind.display_name(),
            role: joinee.role,
            avatar_url: joinee.kind.avatar_url().map(|s| s.to_owned()),
            participation_kind: joinee.kind.participant_kind(),
            joined_at: joinee.joined_at.into(),
            left_at: joinee.left_at.map(Into::into),
            is_room_owner: participant_id_from_uuid(self.info.owner()) == participant_id,
        };
        let data = SharedJson::from(
            serde_json::to_value(data)
                .context("Failed to serialize CoreState")
                .map_err(FatalError)?,
        );
        for peer_id in self.participants.connected().ids() {
            peer_events
                .entry(peer_id)
                .or_default()
                .insert(CORE_MODULE_ID, data.clone());
        }
        Ok(())
    }

    /// Attach core related info about the other participant (`peer`) for the joining participant.
    ///
    /// This will be sent to the joining participant.
    pub(crate) fn join_success_core_peer_data(
        &self,
        peer: ParticipantId,
        state: &ParticipantState,
        peer_data: &mut BTreeMap<ModuleId, SharedJson>,
    ) -> Result<(), FatalError> {
        let data = CoreState {
            display_name: state.kind.display_name(),
            role: state.role,
            avatar_url: state.kind.avatar_url().map(str::to_owned),
            participation_kind: state.kind.participant_kind(),
            joined_at: state.joined_at.into(),
            left_at: state.left_at.map(Into::into),
            is_room_owner: participant_id_from_uuid(self.info.owner()) == peer,
        };
        let data = SharedJson::from(
            serde_json::to_value(data)
                .context("Failed to serialize CoreState")
                .map_err(FatalError)?,
        );
        peer_data.insert(CORE_MODULE_ID, data);

        Ok(())
    }
}

#[must_use]
pub(crate) struct RequestedModuleActions {
    pub messages: RefCell<Vec<ModuleMessage>>,
    pub errors: Vec<(ModuleId, FatalError)>,
    pub timestamp: Timestamp,
    pub room_kind: RoomKind,
    pub origin: EventOrigin,
}

impl RequestedModuleActions {
    pub(crate) fn handle_requested_messages(
        self,
        task: &mut RoomTask<impl SignalingSocket>,
    ) -> Result<(), FatalError> {
        task.handle_module_messages(self.messages, self.room_kind, self.origin, self.timestamp)?;

        for (namespace, err) in self.errors {
            task.handle_fatal_module_error(namespace, self.origin.transaction_id(), err);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::signaling_event::SignalingEvent;
    use opentalk_roomserver_types::{
        connection_id::ConnectionId,
        core::CORE_MODULE_ID,
        device_id::DeviceId,
        join::{connection_info::ConnectionInfo, join_success::JoinSuccess},
        room_info::RoomInfo,
        shared_json::SharedJson,
    };
    use opentalk_types_common::{
        events::MeetingDetails,
        modules::{ModuleId, module_id},
        rooms::RoomId,
        tariffs::TariffResource,
        time::Timestamp,
        users::{DisplayName, UserInfo},
        utils::ExampleData,
    };
    use opentalk_types_signaling::{ModuleData, ParticipantId, Role, SignalingModuleFrontendData};
    use pretty_assertions::assert_eq;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

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
            connection_id: ConnectionId::from_u128(0x1),
            device_id: DeviceId::from_u128(0x2),
            connections: vec![ConnectionInfo {
                connection_id: ConnectionId::from_u128(0x3),
                device_id: DeviceId::from_u128(0x4),
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
            timestamp: Timestamp::unix_epoch(),
            payload: CoreEvent::JoinSuccess(join_success.into()),
            transaction_id: Some(0),
        };

        let produced = serde_json::to_string_pretty(&event).unwrap();
        assert_snapshot!(produced, @r#"
        {
          "namespace": "core",
          "transaction_id": 0,
          "timestamp": "1970-01-01T00:00:00Z",
          "payload": {
            "join_success": {
              "id": "00000000-0000-0000-0000-000000000000",
              "connection_id": "00000000-0000-0000-0000-000000000001",
              "device_id": "00000000-0000-0000-0000-000000000002",
              "connections": [
                {
                  "connection_id": "00000000-0000-0000-0000-000000000003",
                  "device_id": "00000000-0000-0000-0000-000000000004"
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
              "meeting_details": {
                "streaming_links": []
              },
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
          }
        }
        "#);

        let json = json!({
            "namespace": "core",
            "transaction_id": 0,
            "timestamp": "1970-01-01T00:00:00Z",
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
        });

        let _: SignalingEvent<CoreEvent> = serde_json::from_value(json).unwrap();
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
        let mut peer_join_data = BTreeMap::new();
        peer_join_data.insert(
            module_id!("test"),
            SharedJson::from(json!({
                "key": "value"
            })),
        );

        let event = CoreEvent::ParticipantConnected {
            participant_id: ParticipantId::nil(),
            connection_id: ConnectionId::nil(),
            peer_data: peer_join_data,
        };
        let json = serde_json::to_value(&event).unwrap();

        assert_eq!(
            json,
            json!({
                "participant_connected": {
                    "participant_id": "00000000-0000-0000-0000-000000000000",
                    "connection_id": "00000000-0000-0000-0000-000000000000",
                    "peer_data": {
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
