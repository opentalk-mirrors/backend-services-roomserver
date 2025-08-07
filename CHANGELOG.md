# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.5] - 2025-08-07

[0.0.5]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.4...v0.0.5

### 🐛 Bug fixes

- Add missing crate features ([!466](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/466))

## [0.0.4] - 2025-08-07

[0.0.4]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.3...v0.0.4

### 🚀 New features

- (mock) Use explicit RoomParameters for test ([!426](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/426))
- (mock) Better errors when events are not sent ([!426](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/426))
- (mock) Add module init data ([!426](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/426))
- (dui) Store and migrate versioned settings ([!427](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/427))
- (mock) Allow to set the Event in the RoomBuilder ([!390](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/390))
- (shared-folder) Add SharedFolder SignalingModule ([!390](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/390))
- (client) Add SharedFolder event and command types ([!390](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/390))
- Add meeting start, end and timezone to `EventContext` ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- Add joined/left timestamps to participant state ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- Add email to participant state ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- Add `visible()` filter for participant state ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- Add report generation crate ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- (meeting-report) Add meeting-report types ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- (meeting-report) Add meeting-report module ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- (test) Configure logging to also support tracing logs ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (room) Replace `log` with `tracing` ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (chat) Replace `log` with `tracing` ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (ping) Replace `log` with `tracing` ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (polls) Replace `log` with `tracing` ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (shared-folder) Replace `log` with `tracing` ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (roomserver) Replace `log` with `tracing` ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (web-api) Replace `log` with `tracing` ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (signaling) Replace `log` with `tracing` ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- Inter module communication ([!441](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/441))
- (room) Instroduce waiting room ([!412](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/412), [#114](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/114))
- (room) Add core commands ([!450](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/450))
- (moderation) Implement kick participants instruction ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (moderation) Add kick and debrief types ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (moderation) Implement kick and debrief in moderation module ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (client) Add moderation to client library ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (chat) Chunk message history ([!411](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/411))

### 🐛 Bug fixes

- (release) Add descriptions for polls and timer types crates ([!424](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/424))
- (justfile) Recommend correct yq variant when prompting to install ([!423](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/423))
- (dui) Fix layout to show parameter save button ([!427](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/427))
- Shut down modules when exiting room task ([!438](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/438))
- (ci) Add trivy ignore file for alpine ([!439](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/439))
- (ci) Initialize module data in livekit tests ([!446](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/446))
- (room) Close websocket when disconnecting participant ([!450](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/450))
- (room) Second close frame when disconnecting participant ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))

### 🔨 Refactor

- Simplify `ParticipantKind` enum ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- (storage) Include filename in `AssetUploaded` ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- (storage) Allow using the storage api in an async context ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- (room) Move command handling to function ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (room) Introduce functions for joining the room ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (room) Introduce function to execute SignalingModule commands ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- Make `build_join_success` an associated function of `RoomTask` ([!441](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/441))
- (signaling) Allow better chaining of filter and iterator ([!412](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/412))
- Simplify comments in message router ([!450](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/450))
- (room) Rename `result_handle` field/parameter to `result_callback` ([!450](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/450))
- (room) Remove `ModulePeerData` from `JoinedWaitingRoom` event ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (signaling) Allow better chaining of participant id filter ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (room) Rename core commands to instructions ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (room) Make entering from the waiting room a core command/event ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (moderation) Move accept to moderation module ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (room) Improve tracing when performing close handshake ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (room) Make `MessageRouter::remove_connection()` synchronous ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (room) Move timezone to `PublicUserProfile` and make it required ([!456](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/456))

### 📦 Dependencies

- (deps) Update rust crate egui_json_tree to v0.12.1 ([!429](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/429))
- (deps) Update rust crate livekit to v0.7.15 ([!428](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/428))
- (deps) Lock file maintenance ([!434](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/434))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.17.6 ([!430](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/430))
- (deps) Update rust crate egui_json_tree to 0.13.0 ([!435](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/435))
- (deps) Update `opentalk-types-common` to 0.35.3 ([!436](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/436))
- (deps) Lock file maintenance ([!445](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/445))
- (deps) Update rust crate testcontainers to 0.25.0 ([!444](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/444))
- (deps) Downgrade rust crate rustls to 0.23.29 ([!447](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/447))
- (deps) Update rust crate clap to v4.5.42 ([!449](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/449))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.17.7 ([!454](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/454))
- (deps) Update rust crate serde_json to v1.0.142 ([!453](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/453))
- (deps) Update rust crate livekit to v0.7.16 ([!451](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/451))
- (deps) Lock file maintenance ([!458](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/458))
- (deps) Update rust crate ecow to v0.2.6 ([!459](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/459))
- (deps) Update rust crate opentalk-types-common to v0.35.4 ([!460](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/460))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.17.8 ([!461](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/461))
- (deps) Update rust crate opentalk-types-common to v0.35.5 ([!462](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/462))
- (deps) Update rust crate clap to v4.5.43 ([!463](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/463))
- (deps) Update opentalk ([!464](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/464))

### ⚙ Miscellaneous

- Add description to all crates ([!426](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/426))
- (test) Require using the result of received_nothing ([!411](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/411))

### Ci

- (justfile) Make all recipes quiet ([!425](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/425))
- (justfile) Remove shebang from recipes when unnecessary ([!425](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/425))
- (justfile) Tag release ([!425](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/425))
- Update packages of alpine image on each build ([!442](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/442))

### Test

- Allow accessing stored files from `FsStorage` ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- Allow configuring storage quota ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- (meeting-report) Add integration tests for meeting-report module ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- (moderation) Add integration tests for kick and debrief ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))

## [0.0.3] - 2025-07-15

[0.0.3]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.2...v0.0.3

### 🚀 New features

- (client) Add timer types ([!394](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/394))
- (client) Add polls types ([!396](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/396))
- Configure modules with room parameter ([!402](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/402))
- Storage API interface ([!389](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/389))
- (signaling) Filter for moderators ([!413](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/413))

### 🔨 Refactor

- (chat) Remove peer_state directory ([!397](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/397))

### 📦 Dependencies

- (deps) Lock file maintenance ([!401](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/401))
- (deps) Update rust crate config to v0.15.12 ([!405](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/405))
- (deps) Update rust crate clap to v4.5.41 ([!407](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/407))
- (deps) Update rust crate config to v0.15.13 ([!406](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/406))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.18.3 ([!416](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/416))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.17.5 ([!418](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/418))
- (deps) Lock file maintenance ([!415](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/415))
- (deps) Update pre-commit hook pre-commit/pre-commit-hooks to v3.4.0 ([!419](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/419))
- (deps) Update pre-commit hook adrienverge/yamllint to v1.37.1 ([!417](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/417))
- (deps) Update pre-commit hook pre-commit/pre-commit-hooks to v5 ([!420](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/420))
- (deps) Update rust crate egui to 0.32.0 ([!409](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/409))

### ⚙ Miscellaneous

- Remove opentalk-types-signaling-chat dependency ([!397](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/397))

### Ci

- (renovate) Add egui group ([!409](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/409))

## [0.0.2] - 2025-07-04

[0.0.2]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.1...v0.0.2

### 🚀 New features

- (signaling) Add default implementation for `on_loopback_event` ([!377](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/377))
- (e2ee) Introduce e2ee signaling module ([!377](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/377))
- (client) Add e2ee types ([!377](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/377))
- (ping) Remove test commands from ping ([!384](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/384), [#95](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/95))
- (chat) Remove serde feature from chat types crate ([!385](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/385), [#86](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/86))
- (roomserver-client) Remove feature flags to avoid openssl dependency ([!398](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/398))

### 🔨 Refactor

- Break cycle and move test to room crate ([!387](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/387), [#105](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/105))
- Move timer types into roomserver ([!393](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/393))

### 📦 Dependencies

- (deps) Update rust crate http-request-derive-client-reqwest to 0.2.0 ([!391](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/391))
- (deps) Update rust crate reqwest to v0.12.22 ([!386](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/386))
- (deps) Update rust crate tokio to v1.46.0 ([!388](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/388))

### Ci

- Add alpine based image and make it the default ([!392](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/392))

### Test

- (mock) Introduce Dave to the gang ([!377](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/377))

## [0.0.1] - 2025-07-01

[0.0.1]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/tags/v0.0.1

The initial `RoomServer` release!

This release contains the following crates:

- `opentalk-roomserver-types`
- `opentalk-roomserver-signaling`
- `opentalk-roomserver-client`
- `opentalk-roomserver-signaling`
- `opentalk-roomserver-types`
- `opentalk-roomserver-types-chat`
- `opentalk-roomserver-types-livekit`
- `opentalk-roomserver-types-ping`
