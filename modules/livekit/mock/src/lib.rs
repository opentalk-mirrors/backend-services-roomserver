// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::time::Duration;

use anyhow::Context;
use livekit::{
    RoomEvent,
    options::TrackPublishOptions,
    track::{LocalAudioTrack, LocalTrack, TrackSource},
    webrtc::{
        audio_source::native::NativeAudioSource,
        prelude::{AudioFrame, AudioSourceOptions, RtcAudioSource},
    },
};
use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, TestRoomBuilder};
use opentalk_roomserver_types_livekit::LiveKitSettings;
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt as _,
    core::{IntoContainerPort, WaitFor, logs::consumer::logging_consumer::LoggingConsumer},
    runners::AsyncRunner as _,
};
use tokio::sync::mpsc::UnboundedReceiver;

pub const LIVEKIT_PORT: u16 = 7880;
pub const LIVEKIT_KEY: &str = "devkey";
pub const LIVEKIT_SECRET: &str = "devsecret";

pub const ENV_LIVEKIT_HOST: &str = "TEST_ROOMSERVER_LIVEKIT_HOST";
pub const ENV_LIVEKIT_PORT: &str = "TEST_ROOMSERVER_LIVEKIT_PORT";
pub const ENV_LIVEKIT_KEYS: &str = "LIVEKIT_KEYS";

fn livekit_config(port: u16, key: &str, secret: &str) -> Vec<u8> {
    let config_str = format!(
        r#"
---
port: {port}
rtc:
    tcp_port: 7881
    # udp_port: 7882
    port_range_start: 50000
    port_range_end: 60000
    use_external_ip: false
keys:
    {key}: {secret}
logging:
    json: false
    level: info
"#,
    );
    config_str.into_bytes()
}

#[derive(Debug, Default)]
pub struct ContainerGuard {
    _inner: Option<ContainerAsync<GenericImage>>,
}

impl From<ContainerAsync<GenericImage>> for ContainerGuard {
    fn from(value: ContainerAsync<GenericImage>) -> Self {
        Self {
            _inner: Some(value),
        }
    }
}

pub async fn build_livekit_room() -> (ContainerGuard, TestRoomBuilder, String) {
    let (livekit_container, settings) = if let Ok(livekit_host) = std::env::var(ENV_LIVEKIT_HOST) {
        let livekit_container = ContainerGuard { _inner: None };
        let host = livekit_host;
        let host_port = livekit_port_from_env().unwrap_or(LIVEKIT_PORT);
        let url = format!("http://{host}:{host_port}");
        let (api_key, api_secret) = livekit_key_from_env()
            .unwrap_or_else(|| (LIVEKIT_KEY.to_string(), LIVEKIT_SECRET.to_string()));

        let settings = LiveKitSettings {
            api_key,
            api_secret,
            public_url: url.clone(),
            service_url: url.clone(),
        };
        (livekit_container, settings)
    } else {
        let (container, host, host_port) = build_from_testcontainer().await;
        let url = format!("http://{host}:{host_port}");

        (
            container.into(),
            LiveKitSettings {
                api_key: LIVEKIT_KEY.to_string(),
                api_secret: LIVEKIT_SECRET.to_string(),
                public_url: url.clone(),
                service_url: url.clone(),
            },
        )
    };

    let room = TestRoom::builder()
        .register_module::<LiveKitModule>()
        .add_init_module_data(&settings)
        .unwrap();
    (livekit_container, room, settings.service_url)
}

fn livekit_port_from_env() -> Option<u16> {
    let port = std::env::var(ENV_LIVEKIT_PORT).ok()?;
    Some(port.parse().expect("Livekit port was invalid"))
}

fn livekit_key_from_env() -> Option<(String, String)> {
    let keys = std::env::var(ENV_LIVEKIT_KEYS).ok()?;
    let keys = keys.split("\n").next()?;
    let (key, secret) = keys.split_once(':')?;
    Some((key.trim().to_string(), secret.trim().to_string()))
}

async fn build_from_testcontainer() -> (ContainerAsync<GenericImage>, String, u16) {
    let livekit_container = GenericImage::new("livekit/livekit-server", "latest")
        .with_exposed_port(LIVEKIT_PORT.tcp())
        .with_wait_for(WaitFor::message_on_stderr("starting LiveKit server"))
        .with_network("bridge")
        .with_copy_to(
            "/livekit.yaml",
            livekit_config(LIVEKIT_PORT, LIVEKIT_KEY, LIVEKIT_SECRET),
        )
        .with_cmd([
            "--config",
            "/livekit.yaml",
            "--dev",
            "--node-ip",
            "127.0.0.1",
        ])
        .with_log_consumer(LoggingConsumer::new())
        .start()
        .await
        .unwrap();
    let host = livekit_container.get_host().await.unwrap().to_string();
    let host_port = livekit_container
        .get_host_port_ipv4(LIVEKIT_PORT)
        .await
        .unwrap();
    (livekit_container, host, host_port)
}

// since all integration tests load this module separately, it will be flagged
// as unused in some of them.
pub async fn publish_audio(
    room: &livekit::Room,
    room_events: &mut UnboundedReceiver<RoomEvent>,
) -> anyhow::Result<LocalAudioTrack> {
    tracing::info!("Try publishing audio");
    const SAMPLE_RATE: u32 = 48000;
    const NUM_CHANNELS: u32 = 2;
    const SAMPLES_PER_CHANNEL: u32 = 480;

    let source =
        NativeAudioSource::new(AudioSourceOptions::default(), SAMPLE_RATE, NUM_CHANNELS, 10);

    let audio_track =
        LocalAudioTrack::create_audio_track("file", RtcAudioSource::Native(source.clone()));
    room.local_participant()
        .publish_track(
            LocalTrack::Audio(audio_track.clone()),
            TrackPublishOptions {
                source: TrackSource::Microphone,
                ..Default::default()
            },
        )
        .await?;
    tracing::info!("Try capture audio frame");

    let audio_frame = AudioFrame::new(SAMPLE_RATE, NUM_CHANNELS, SAMPLES_PER_CHANNEL);
    source.capture_frame(&audio_frame).await?;

    tracing::info!("Audio frame captured");

    loop {
        let event = tokio::time::timeout(Duration::from_secs(1), room_events.recv())
            .await?
            .context("Failed to receive track published event")?;

        tracing::info!("Received LiveKit event: {:?}", event);
        if let RoomEvent::LocalTrackPublished { track, .. } = event
            && track.sid() == audio_track.sid()
        {
            break;
        }
    }

    tokio::time::timeout(Duration::from_secs(1), room_events.recv())
        .await
        .context("Failed to receive participant update")?;

    Ok(audio_track)
}
