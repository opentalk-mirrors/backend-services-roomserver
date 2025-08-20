// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use livekit::{
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

pub const LIVEKIT_PORT: u16 = 7880;
pub const LIVEKIT_KEY: &str = "devkey";
pub const LIVEKIT_SECRET: &str = "devsecret";

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

pub async fn build_livekit_room() -> (ContainerAsync<GenericImage>, TestRoomBuilder, String) {
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
    let host = livekit_container.get_host().await.unwrap();
    let host_port = livekit_container
        .get_host_port_ipv4(LIVEKIT_PORT)
        .await
        .unwrap();

    let url = format!("http://{host}:{host_port}");

    let room = TestRoom::builder()
        .register_module::<LiveKitModule>()
        .add_init_module_data(&LiveKitSettings {
            api_key: LIVEKIT_KEY.to_string(),
            api_secret: LIVEKIT_SECRET.to_string(),
            public_url: url.clone(),
            service_url: url.clone(),
        })
        .unwrap();
    (livekit_container, room, url)
}

// since all integration tests load this module separately, it will be flagged
// as unused in some of them.
#[allow(dead_code)]
pub async fn publish_audio(room: &livekit::Room) -> anyhow::Result<()> {
    tracing::info!("Try publishing audio");
    const SAMPLE_RATE: u32 = 48000;
    const NUM_CHANNELS: u32 = 2;
    const SAMPLES_PER_CHANNEL: u32 = 480;

    let source =
        NativeAudioSource::new(AudioSourceOptions::default(), SAMPLE_RATE, NUM_CHANNELS, 10);

    let track = LocalAudioTrack::create_audio_track("file", RtcAudioSource::Native(source.clone()));
    room.local_participant()
        .publish_track(
            LocalTrack::Audio(track),
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
    Ok(())
}
