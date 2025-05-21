// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::BTreeMap, convert::Infallible, fmt::Debug, marker::PhantomData, sync::Arc,
    time::Duration,
};

use anyhow::Context;
use opentalk_roomserver_common::settings::Settings;
use opentalk_roomserver_types::{breakout_id::BreakoutId, connection_id::ConnectionId};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::{ParticipantId, SignalingModuleFrontendData};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::value::{RawValue, to_raw_value};

use super::module_context::ModuleContext;
use crate::{breakout::BreakoutRoom, participant_state::ParticipantState};

/// The trait that defines a signaling module
///
/// Implementors can be added as a module to the room task. The room task will forward signaling events to the module
/// with the corresponding [`SignalingModule::NAMESPACE`]. All event calls are handled in sequence on the same task.
/// Signaling modules are expected to spawn separate tasks when compute intense or long-running operations need to be
/// executed (See [`SignalingModule::Loopback`] for more details).
pub trait SignalingModule: Send + Sync + Sized {
    /// The unique namespace for the module
    ///
    /// This is used as a general identifier to dispatch incoming signaling messages to the correct module.
    const NAMESPACE: ModuleId;

    /// The incoming websocket payload which is received as in [`SignalingModule::on_websocket_message`]
    type Incoming: for<'de> Deserialize<'de> + Send + CreateReplica<Self::Outgoing>;

    /// The outgoing websocket payload that is sent to the clients
    type Outgoing: Serialize + PartialEq + Debug + From<Self::Error> + Send;

    /// Internal result type for asynchronous tasks
    ///
    /// These are received in the [`SignalingModule::on_loopback_event`] when an asynchronous task created by
    /// the module finishes.
    ///
    /// Tasks can be created with [`ModuleContext::spawn`] or [`ModuleContext::spawn_blocking`].
    type Loopback: Send + 'static;

    /// Namespaced data that can be attached to a participants `JoinSuccess` message
    type JoinInfo: SignalingModuleFrontendData + Clone + Send;

    /// Namespaced data that can be attached to the `Joined` message
    ///
    /// When a participant connects they trigger a `Joined` event for all other participants in the conference. Modules
    /// can append this type to the message to communicate module specific state of a new participant to the other
    /// participants.
    type PeerJoinInfo: Serialize + Send + 'static;

    /// The non-fatal error that can be returned from signaling module event handlers
    ///
    /// Is converted into a websocket event and returned to the command issuing participant
    ///
    /// Use [`Infallible`] if there is no error case.
    type Error: ModuleError;

    /// Creates an instance of the interface to access the module
    fn init(init_data: SignalingModuleInitData) -> Option<Self>;

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        is_first_connection: bool,
    ) -> Result<JoinInfo<Self>, SignalingModuleError<Self::Error>>;

    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>>;

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        connection_id: ConnectionId,
        content: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>>;

    #[allow(unused_variables)]
    fn on_breakout_start(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        rooms: &[BreakoutRoom],
        duration: Option<Duration>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn on_breakout_switch(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        old_room: Option<BreakoutId>,
        new_room: Option<BreakoutId>,
    ) -> Result<BTreeMap<ConnectionId, Self::JoinInfo>, SignalingModuleError<Self::Error>> {
        Ok(BTreeMap::new())
    }

    #[allow(unused_variables)]
    fn on_breakout_stop(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>>;

    /// Destroy the module and remove all associated resources
    ///
    /// Long running tasks must be spawned in a separate task
    fn destroy(self) {}
}

pub trait CreateReplica<T> {
    fn replicate(&self) -> Option<T>;
}

pub struct JoinInfo<M: SignalingModule> {
    /// Module specific data that will be attached to the participants `JoinedSuccess` message
    pub join_success: Option<M::JoinInfo>,

    /// Module specific data that will be attached to other participants `Joined` message
    pub peer: PeerJoinInfoMap<M>,
}

impl<M: SignalingModule> Default for JoinInfo<M> {
    fn default() -> Self {
        Self {
            join_success: Default::default(),
            peer: Default::default(),
        }
    }
}

/// A map of participants and their [`SignalingModule::PeerJoinInfo`]
///
/// When a `Joined` message is sent to a participant, the participants associated [`SignalingModule::PeerJoinInfo`] will
/// be attached to it.
pub struct PeerJoinInfoMap<M: SignalingModule> {
    pub map: BTreeMap<ParticipantId, SharedRawJson>,
    _m: PhantomData<M>,
}

impl<M: SignalingModule> PeerJoinInfoMap<M> {
    pub fn new(map: BTreeMap<ParticipantId, SharedRawJson>) -> Self {
        Self {
            map,
            _m: PhantomData,
        }
    }
}

impl<M: SignalingModule> Default for PeerJoinInfoMap<M> {
    fn default() -> Self {
        Self {
            map: Default::default(),
            _m: PhantomData,
        }
    }
}

impl<M: SignalingModule> PeerJoinInfoMap<M> {
    /// Attach the same [`PeerJoinInfo`](SignalingModule::PeerJoinInfo) for all participants
    pub fn insert_for_all(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        info: M::PeerJoinInfo,
    ) -> Result<(), FatalError> {
        self.insert_for_matching(ctx, info, |_, _| true)
    }

    /// Attach [`PeerJoinInfo`](SignalingModule::PeerJoinInfo) to a single participant
    pub fn insert(
        &mut self,
        participant_id: ParticipantId,
        info: M::PeerJoinInfo,
    ) -> Result<(), FatalError> {
        let raw_value = SharedRawJson::from(
            to_raw_value(&info)
                .with_context(|| {
                    format!(
                        "Failed to serialize PeerJoinInfo for module '{}'",
                        M::NAMESPACE
                    )
                })
                .map_err(FatalError)?,
        );

        self.map.insert(participant_id, raw_value);

        Ok(())
    }

    /// Attach [`PeerJoinInfo`](SignalingModule::PeerJoinInfo) for all participants that match the given filter
    pub fn insert_for_matching<F>(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        info: M::PeerJoinInfo,
        mut filter: F,
    ) -> Result<(), FatalError>
    where
        F: FnMut(ParticipantId, &ParticipantState) -> bool,
    {
        // Lazily serialize the PeerJoinInfo into a json string
        let mut raw_value: Option<SharedRawJson> = None;

        for (participant_id, state) in ctx.participants.connected().iter() {
            if !filter(*participant_id, state) {
                continue;
            }

            // Serialize the PeerJoinInfo if it hasn't already
            let raw_value = match &mut raw_value {
                Some(raw_value) => raw_value,
                None => raw_value.insert(SharedRawJson::from(
                    to_raw_value(&info)
                        .with_context(|| {
                            format!(
                                "Failed to serialize PeerJoinInfo for module '{}'",
                                M::NAMESPACE
                            )
                        })
                        .map_err(FatalError)?,
                )),
            };

            self.map.insert(*participant_id, raw_value.clone());
        }

        Ok(())
    }
}

/// Data that a signaling module might require to initialize
#[derive(Clone, Debug)]
pub struct SignalingModuleInitData {
    /// The roomserver settings
    pub settings: Arc<Settings>,
}

/// Marker trait to allow us to convert the [`SignalingModule::Error`] into a [`SignalingModuleError`]
pub trait ModuleError: Debug + Send {}

impl ModuleError for Infallible {}

/// The error type returned by signaling module event handlers
#[derive(Debug)]
pub enum SignalingModuleError<E> {
    /// An non-fatal internal error occurred
    Internal(anyhow::Error),
    /// A fatal error occurred.
    ///
    /// This is considered to be unrecoverable, the module will be flagged as broken and deactivated
    Fatal(FatalError),
    /// The module specific error
    ///
    /// Is turned into a websocket message and returned to the command issuing participant
    Module(E),
}

impl<E> From<anyhow::Error> for SignalingModuleError<E> {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err)
    }
}

impl<E: ModuleError> From<E> for SignalingModuleError<E> {
    fn from(err: E) -> Self {
        Self::Module(err)
    }
}

#[derive(Debug)]
pub struct FatalError(pub anyhow::Error);

impl<E> From<FatalError> for SignalingModuleError<E> {
    fn from(err: FatalError) -> Self {
        Self::Fatal(err)
    }
}

/// Type to deal with opaque JSON values.
///
/// Some scenarios require sending the same value to a large amount of participants,
/// which is why the value is reference counted and therefore cheap to clone.
#[derive(Debug, Clone)]
pub struct SharedRawJson {
    inner: Arc<RawValue>,
}

impl From<Box<RawValue>> for SharedRawJson {
    fn from(value: Box<RawValue>) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl Serialize for SharedRawJson {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        RawValue::serialize(&*self.inner, serializer)
    }
}

impl<'de> Deserialize<'de> for SharedRawJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <Box<RawValue>>::deserialize(deserializer).map(Self::from)
    }
}
