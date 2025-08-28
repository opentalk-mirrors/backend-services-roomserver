// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{ModuleJoinData, NoOp, SignalingModule, SignalingModuleInitData},
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId, signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_e2ee::{
    E2EE_MODULE_ID, E2eeCommand, E2eeError, E2eeEvent, Invite, MlsMessages,
};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;

#[derive(Debug)]
pub struct E2eeModule;

impl SignalingModule for E2eeModule {
    const NAMESPACE: ModuleId = E2EE_MODULE_ID;

    type Incoming = E2eeCommand;

    type Outgoing = E2eeEvent;

    type Internal = NoOp;

    type Loopback = ();

    type JoinInfo = ();

    type PeerJoinInfo = ();

    type Error = E2eeError;

    fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self)
    }

    fn on_participant_joined(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        _participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        Ok(ModuleJoinData::default())
    }

    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        ctx.send_ws_message(
            ctx.participants.connected().room(ctx.room).ids(),
            E2eeEvent::Disconnect {
                participant_id,
                connection_id,
            },
        )?;
        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        _sender: ParticipantId,
        connection_id: ConnectionId,
        payload: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match payload {
            E2eeCommand::Invite(invite) => self.publish_invite(ctx, connection_id, invite),
            E2eeCommand::Message(message) => self.forward_message(ctx, connection_id, message),
        }
    }
}

impl E2eeModule {
    fn publish_invite(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ConnectionId,
        invite: Invite,
    ) -> Result<(), SignalingModuleError<<Self as SignalingModule>::Error>> {
        self.ensure_valid_invite(ctx, &invite)?;

        // send MLS message to all but the invitee and sender
        ctx.send_ws_message_to_connections(
            ctx.participants
                .connected()
                .room(ctx.room)
                .connection_ids()
                .filter(|&p| p != sender && p != invite.invitee),
            E2eeEvent::MlsMessages(invite.mls_messages),
        )?;

        ctx.send_ws_message_to_connections(
            [invite.invitee],
            E2eeEvent::Welcome(invite.welcome_message),
        )?;

        Ok(())
    }

    fn ensure_valid_invite(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        invite: &Invite,
    ) -> Result<(), SignalingModuleError<<Self as SignalingModule>::Error>> {
        if !invite.welcome_message.is_valid() || !invite.mls_messages.is_valid() {
            return Err(E2eeError::InvalidInvite.into());
        }
        self.ensure_valid_connection(ctx, &invite.invitee)
    }

    fn ensure_valid_connection(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        connection: &ConnectionId,
    ) -> Result<(), SignalingModuleError<<Self as SignalingModule>::Error>> {
        if ctx
            .participants
            .connected()
            .room(ctx.room)
            .connection_ids()
            .any(|c| &c == connection)
        {
            Ok(())
        } else {
            Err(E2eeError::InvalidParticipantTarget.into())
        }
    }

    fn forward_message(
        &self,
        ctx: &mut ModuleContext<'_, E2eeModule>,
        connection_id: ConnectionId,
        message: MlsMessages,
    ) -> Result<(), SignalingModuleError<<Self as SignalingModule>::Error>> {
        ctx.send_ws_message_to_connections(
            ctx.participants
                .connected()
                .room(ctx.room)
                .connection_ids()
                .filter(|&c| connection_id != c),
            E2eeEvent::MlsMessages(message),
        )?;
        Ok(())
    }
}
