// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_orchestrator_client::{
    Metrics, RegisterRoomserver, RegisterType, client::StateProvider,
};
use opentalk_roomserver_web_api::v1::signaling::websocket::SignalingSocket;
use sysinfo::{CpuRefreshKind, RefreshKind, System};

use crate::RoomTaskRegistry;

pub struct OrchestratorStateProvider<S: SignalingSocket + 'static> {
    registry: RoomTaskRegistry<S>,
    system: System,
}

impl<S: SignalingSocket + 'static> OrchestratorStateProvider<S> {
    pub fn new(registry: RoomTaskRegistry<S>) -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing().with_cpu_usage()),
        );
        system.refresh_cpu_usage();

        Self { registry, system }
    }

    fn calculate_load(&mut self) -> u8 {
        self.system.refresh_cpu_usage();
        let cpu_usage = self.system.global_cpu_usage();

        (cpu_usage as u8).min(100)
    }
}

#[async_trait]
impl<S: SignalingSocket + 'static> StateProvider for OrchestratorStateProvider<S> {
    async fn register_type(&mut self) -> opentalk_orchestrator_client::RegisterType {
        let rooms = self.registry.room_ids().await;

        RegisterType::Roomserver(RegisterRoomserver { rooms })
    }

    async fn metrics(&mut self) -> Metrics {
        Metrics {
            load: self.calculate_load(),
            accepting_jobs: true,
        }
    }
}
