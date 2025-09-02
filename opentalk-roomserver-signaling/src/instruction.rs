// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_signaling::ParticipantId;

#[derive(Debug)]
pub enum Instruction {
    Kick { participants: Vec<ParticipantId> },
    Ban { participant: ParticipantId },
    BanWaiting { participant: ParticipantId },
    MoveToWaitingRoom { participant: ParticipantId },
}
