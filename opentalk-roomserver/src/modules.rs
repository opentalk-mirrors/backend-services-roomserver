// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_module_automod::AutomodModule;
use opentalk_roomserver_module_chat::ChatModule;
use opentalk_roomserver_module_e2ee::E2eeModule;
use opentalk_roomserver_module_echo::EchoModule;
use opentalk_roomserver_module_legal_vote::LegalVoteModule;
use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_module_meeting_notes::MeetingNotesModule;
use opentalk_roomserver_module_meeting_report::MeetingReportModule;
use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_module_polls::PollsModule;
use opentalk_roomserver_module_raise_hands::RaiseHandsModule;
use opentalk_roomserver_module_recording::RecordingModule;
use opentalk_roomserver_module_shared_folder::SharedFolderModule;
use opentalk_roomserver_module_subroom_audio::SubroomAudioModule;
use opentalk_roomserver_module_timer::TimerModule;
use opentalk_roomserver_module_training_participation_report::TrainingParticipationReportModule;
use opentalk_roomserver_module_whiteboard::WhiteboardModule;
use opentalk_roomserver_room::ModuleRegistry;

/// Initialize the registry with all modules that are available for meetings
pub fn setup_registry() -> ModuleRegistry {
    let mut module_registry = ModuleRegistry::new();
    module_registry.add_module::<AutomodModule>();
    module_registry.add_module::<ChatModule>();
    module_registry.add_module::<E2eeModule>();
    module_registry.add_module::<LegalVoteModule>();
    module_registry.add_module::<LiveKitModule>();
    module_registry.add_module::<MeetingNotesModule>();
    module_registry.add_module::<MeetingReportModule>();
    module_registry.add_module::<ModerationModule>();
    module_registry.add_module::<RecordingModule>();
    module_registry.add_module::<EchoModule>();
    module_registry.add_module::<PollsModule>();
    module_registry.add_module::<SharedFolderModule>();
    module_registry.add_module::<SubroomAudioModule>();
    module_registry.add_module::<TimerModule>();
    module_registry.add_module::<TrainingParticipationReportModule>();
    module_registry.add_module::<RaiseHandsModule>();
    module_registry.add_module::<WhiteboardModule>();
    module_registry
}
