// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeMap;

use opentalk_roomserver_signaling::signaling_module::{FatalError, SharedRawJson};
use opentalk_roomserver_types::client_parameters::{ClientKind, ClientParameters};
use opentalk_types_common::{
    events::{EventInfo, MeetingDetails},
    modules::{module_id, ModuleId},
};
use opentalk_types_signaling::{ModuleData, ModulePeerData, Participant, ParticipantId, Role};
use opentalk_types_signaling_control::{event::JoinSuccess, room::RoomInfo};
use serde::{Deserialize, Serialize};

use super::{handle_fatal_module_error, Modules};
use crate::room::{
    message_router::CloseReason,
    signaling::{dyn_module_context::DynModuleContext, DynBroadcastEvent},
};

pub const NAMESPACE: ModuleId = module_id!("core");

/// Outgoing websocket messages in the core namespace
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreEvent {
    /// Message sent to a participant on a successful join
    JoinSuccess(JoinSuccess),

    /// Broadcast message sent to all participants when a new participant has joined
    ParticipantJoined {
        participant_id: ParticipantId,
        peer_join_info: BTreeMap<ModuleId, SharedRawJson>,
    },

    /// Broadcast message sent to all participants when a participant disconnected
    ParticipantDisconnected {
        participant_id: ParticipantId,
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

/// A participant connected to the conference
///
/// Sends the [`CoreEvent::JoinSuccess`] to the joining participant and notifies other participants with the
/// [`CoreEvent::ParticipantJoined`] message.
pub(super) async fn participant_joined(
    ctx: &mut DynModuleContext<'_>,
    participant_id: ParticipantId,
    modules: &mut Modules,
    client_parameters: ClientParameters,
) -> Result<(), FatalError> {
    let mut module_data = ModuleData::new();
    let mut peer_module_data = BTreeMap::new();

    broadcast_event_to_modules(
        ctx,
        modules,
        DynBroadcastEvent::Joined {
            participant_id,
            module_data: &mut module_data,
            peer_module_data: &mut peer_module_data,
        },
    )
    .await;

    let join_success_msg = build_join_success(ctx, participant_id, client_parameters, module_data);
    ctx.send_ws_message(
        participant_id,
        NAMESPACE,
        CoreEvent::JoinSuccess(join_success_msg),
    )
    .await?;

    for (peer, module_data) in peer_module_data {
        let joined_msg = CoreEvent::ParticipantJoined {
            participant_id,
            peer_join_info: module_data,
        };

        ctx.send_ws_message(peer, NAMESPACE, joined_msg).await?;
    }

    ctx.participants.insert(participant_id);

    Ok(())
}

/// Inform modules that the participant has left the conference and broadcast [`CoreEvent::ParticipantDisconnected`]
/// to all participants
pub(super) async fn participant_disconnected(
    ctx: &mut DynModuleContext<'_>,
    reason: DisconnectReason,
    modules: &mut Modules,
) {
    broadcast_event_to_modules(ctx, modules, DynBroadcastEvent::Left(ctx.participant_id)).await;

    let content = CoreEvent::ParticipantDisconnected {
        participant_id: ctx.participant_id,
        reason,
    };

    ctx.broadcast_ws_message(NAMESPACE, content)
        .await
        .expect("CoreEvent::ParticipantDisconnected must be serializable");
}

fn build_join_success(
    ctx: &DynModuleContext<'_>,
    participant_id: ParticipantId,
    client_parameters: ClientParameters,
    module_data: ModuleData,
) -> JoinSuccess {
    let participants = ctx
        .participants
        .iter()
        .map(|id| Participant {
            id: *id,
            module_data: ModulePeerData::new(),
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

/// Broadcast the [`DynBroadcastEvent`] to all modules
async fn broadcast_event_to_modules(
    ctx: &mut DynModuleContext<'_>,
    modules: &mut Modules,
    mut event: DynBroadcastEvent<'_>,
) {
    let mut errors = Vec::new();
    for (namespace, module) in modules.iter_mut() {
        if let Err(err) = module.on_broadcast_event(ctx, &mut event).await {
            errors.push((namespace.clone(), err));
        }
    }

    for (namespace, err) in errors {
        handle_fatal_module_error(ctx, &mut *modules, namespace, err).await;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use opentalk_roomserver_signaling::signaling_module::SharedRawJson;
    use opentalk_types_common::{
        modules::module_id,
        rooms::RoomId,
        tariffs::TariffResource,
        users::{DisplayName, UserInfo},
        utils::ExampleData,
    };
    use opentalk_types_signaling::{ModuleData, ParticipantId, Role};
    use opentalk_types_signaling_control::{event::JoinSuccess, room::RoomInfo};
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
        let event = CoreEvent::JoinSuccess(join_success);
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
                            "media": {
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

        let event = CoreEvent::ParticipantJoined {
            participant_id: ParticipantId::nil(),
            peer_join_info,
        };
        let json = serde_json::to_value(&event).unwrap();

        assert_eq!(
            json,
            json!({
              "participant_joined": {
                "participant_id": "00000000-0000-0000-0000-000000000000",
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
            reason: DisconnectReason::ConnectionLost,
        };

        let json = serde_json::to_value(&event).unwrap();

        assert_eq!(
            json,
            json!({
              "participant_disconnected": {
                "participant_id": "00000000-0000-0000-0000-000000000000",
                "reason": "connection_lost"
              }
            })
        );
    }
}
