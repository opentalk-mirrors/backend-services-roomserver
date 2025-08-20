// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to signaling events in the `subroom_audio` namespace

mod error;
mod participants_invited;
mod subroom_audio_events;
mod whisper_accepted;
mod whisper_group_outgoing;
mod whisper_invite;
mod whisper_participant_info;
mod whisper_token;

pub use error::SubroomAudioError;
pub use participants_invited::ParticipantsInvited;
pub use subroom_audio_events::SubroomAudioEvent;
pub use whisper_accepted::WhisperAccepted;
pub use whisper_group_outgoing::WhisperGroupOutgoing;
pub use whisper_invite::WhisperInvite;
pub use whisper_participant_info::WhisperParticipantInfo;
pub use whisper_token::WhisperToken;
