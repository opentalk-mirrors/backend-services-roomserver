# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.13] - 2025-11-06

[0.0.13]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.12...v0.0.13

### 🚀 New features

- Allow for the room idle timeout to be configurable ([!680](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/680))
- (dui) Add moderator tools plugin ([!617](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/617))
- Add controller asset storage ([!617](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/617))

### 🐛 Bug fixes

- Remove rooms from the registry when they are done ([!680](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/680))

### 🔨 Refactor

- (test) Rename all `serde_tests` modules to `tests` ([!678](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/678))
- Replace `Duration::from_secs` with `from_mins`/`from_hours` where it makes sense ([!683](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/683))
- Join blocked event to include reason field ([!690](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/690))
- (shared-folder) Remove event error variant ([!691](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/691))
- (signaling) Keep consistent wording in asset API ([!617](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/617))
- Replace chunk uploading with stream implementation ([!617](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/617))
- Rename `StorageError::QuotaReached` to `QuotaExceeded` ([!617](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/617))

### 📦 Dependencies

- (deps) Update rust crate livekit to v0.7.24 ([!658](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/658))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.19.2 ([!674](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/674))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.19.3 ([!676](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/676))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.19.4 ([!679](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/679))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.91.0 ([!682](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/682))
- (deps) Update livekit/livekit-server docker tag to v1.9.3 ([!685](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/685))
- (deps) Lock file maintenance ([!687](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/687))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.19.5 ([!689](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/689))
- (deps) Update opentalk ([!692](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/692))

### ⚙ Miscellaneous

- (shared-folder) Adhere to module directory naming convention ([!691](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/691))
- (client) Add additional tracing spans to improve error tracking ([!617](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/617))
- (meeting-report) Log errors before loosing context ([!617](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/617))

### Ci

- Add doctests ([!681](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/681))

### Test

- (polls) Migrate polls serialization tests to insta ([!684](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/684))
- (polls) Add missing (de)serialization tests ([!684](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/684))
- Cover ControllerAssetStorage ([!617](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/617))

## [0.0.12] - 2025-10-29

[0.0.12]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.11...v0.0.12

### 📦 Dependencies

- (deps) Update pre-commit hook fsfe/reuse-tool to v6.2.0 ([!669](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/669))
- (deps) Update opentalk ([!670](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/670))
- (deps) Remove unnecessary `vergen` and `vergen-gix` dependencies ([!672](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/672))

### ⚙ Miscellaneous

- Set `opentalk-roomserver` as default member ([!673](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/673))

## [0.0.11] - 2025-10-27

[0.0.11]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.10...v0.0.11

### 🚀 New features

- Attach the participants breakout room to the `ParticipantJoined` event data ([!634](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/634), [#149](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/149))
- Enforce room participant limit quota ([!636](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/636), [#146](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/146))
- (chat) Message rate limit ([!578](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/578))
- (room) Graceful room shutdown ([!650](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/650), [#150](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/150))
- (api) Require version specifier for API endpoints ([!663](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/663), [#157](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/157))

### 🐛 Bug fixes

- (just) `mktemp --suffix` doesn't work on MacOS ([!623](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/623))
- (chat) Include breakout chat history on join ([!635](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/635), [#151](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/151))
- (core) Tag `CoreEvent` and enforce snake_case for serialization ([!637](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/637))
- (docs) Wrong status code when request body could not be parsed for PUT `/room/{room_id}/token` ([!656](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/656))
- (client) Remove unnecessary `tokio` and `url` dev dependencies ([!660](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/660))
- (api) Include status code in reply when room parameters are missing ([!660](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/660))
- (docs) Document all possible errors for put_room and request_token ([!660](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/660))

### 🔨 Refactor

- (report-generation) Remove obsolete error variants ([!625](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/625))
- (report-generation) Use `thiserror` ([!625](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/625))
- (polls) Move `Poll` struct from types to module ([!641](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/641))
- Loosen trait bound on `ModuleContext::loopback_after()` ([!641](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/641))
- (timer) Simplify filtering participants for room ([!641](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/641))
- Expose the RoomBackend for the orchestrator ([!648](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/648), [#154](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/154))
- (web-api) Return HTTP 422 for PUT `/room/{room_id}/token` when `RoomParameters` are missing ([!656](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/656), [#136](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/136))
- (client) Remove `http-request-derive` and improve error handling ([!660](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/660))
- (meeting-notes) Use loopback task for deleting pads ([!650](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/650))

### 📦 Dependencies

- (deps) Update pre-commit hook alessandrojcm/commitlint-pre-commit-hook to v9.23.0 ([!624](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/624))
- (deps) Lock file maintenance ([!628](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/628))
- (deps) Update rust crate pdf-extract to 0.10 ([!626](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/626))
- (deps) Update pre-commit hook fsfe/reuse-tool to v6 ([!630](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/630))
- (deps) Update pre-commit hook fsfe/reuse-tool to v6.1.2 ([!631](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/631))
- (deps) Update opentelemetry crates ([!622](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/622))
- (deps) Lock file maintenance ([!639](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/639))
- (deps) Update egui ([!638](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/638))
- (deps) Update livekit ([!640](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/640))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.18.6 ([!647](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/647))
- (deps) Update rust crate half to 2.7.1 ([!641](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/641))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.18.7 ([!649](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/649))
- (deps) Update livekit/livekit-server docker tag to v1.9.2 ([!651](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/651))
- (deps) Lock file maintenance ([!652](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/652))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.18.8 ([!653](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/653))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.18.9 ([!657](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/657))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.19.0 ([!659](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/659))
- (deps) Update opentalk ([!654](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/654))
- (deps) Update typst rust crates ([!665](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/665))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.19.1 ([!666](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/666))
- (deps) Lock file maintenance ([!667](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/667))

### Ci

- Do not include a default config in the containers ([!629](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/629))
- (renovate) Add group rule for typst ([!665](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/665))

### Test

- (mock) Increase socket timeout to 3 seconds ([!633](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/633))
- (chat) Add serialization tests for breakout messages ([!635](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/635))
- (core) Unify `CoreEvent` serialization tests ([!637](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/637))
- (client) Add integration tests for client crate ([!660](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/660))
- (meeting-notes) Pin etherpad container for local tests to v2.0.2 ([!650](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/650))

## [0.0.10] - 2025-09-30

[0.0.10]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.9...v0.0.10

### 🚀 New features

- (core) Add module resources interface ([!556](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/556))

### 🔨 Refactor

- Organize storage module locations ([!556](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/556))

### 📦 Dependencies

- (deps) Update rust crate opentalk-etherpad-client to 0.4 ([!616](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/616))
- (deps) Update opentalk ([!618](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/618))

### ⚙ Miscellaneous

- Switch to internal kaniko image ([!587](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/587))

## [0.0.9] - 2025-09-29

[0.0.9]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.8...v0.0.9

### 🚀 New features

- (types) Add retain function to `ModuleSettings` ([!613](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/613))

### 🐛 Bug fixes

- (room) Use `ParticipantState.display_name` in `ParticipantJoined` if participant already exists ([!604](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/604))
- (room) `CoreCommand::EnterRoom` while already in room returns `SignalingError::UnknownNamespace` ([!612](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/612), [#147](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/147))

### 🔨 Refactor

- (room) Lower trace level for unsupported core commands ([!612](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/612))

### ⚙ Miscellaneous

- (justfile) Update frontend api version number with `set-version` ([!614](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/614))

## [0.0.8] - 2025-09-29

[0.0.8]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.7...v0.0.8

### 🚀 New features

- (whiteboard) Add whiteboard signaling module ([!569](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/569), [#82](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/82))
- Include public url in token response ([!595](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/595), [#145](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/145))
- (roomserver) Per-connection command streams to enable websocket rate limiting ([!586](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/586))
- Allow to configure the asset storage via RoomParameter ([!598](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/598))
- (siganling) Add context information to `StorageProvider` ([!598](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/598))

### 🐛 Bug fixes

- (client) `SignalingModuleEvent::namespace()` returns the wrong id for subroom audio ([!576](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/576))
- Pass correct participant id for core peer data ([!594](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/594))
- (room) Send `JoinedWaitingRoom` event when a participant is moved to the waiting room ([!602](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/602))
- (room) Participants' `in_waiting_room` state is not reset when joining the conference ([!603](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/603))

### 📚 Documentation

- (echo) Document echo types ([!600](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/600))

### 🔨 Refactor

- Apply `clippy::pedantic` lints ([!575](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/575))
- Allow signaling modules to store files when being destroyed ([!569](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/569))
- Replace `ModuleData` with `ModuleSettings` in `RoomParameters` ([!585](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/585), [#144](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/144))
- (automod) Inline structs in event ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (chat) Inline structs in command & event ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (chat) Rename chat module error to `ChatError` ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (e2ee) Inline structs in command ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (meeting-report) Inline structs in event ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (moderation) Inline structs in command & event ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (polls) Inline structs in command & event ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (raise-hands) Inline structs in command ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (subroom-audio) Inline structs in command & event ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (timer) Inline structs in command & event ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (echo) Remove event error variant ([!600](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/600))
- (echo) Reexport command and event ([!600](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/600))
- (room) Separate conference and waiting room commands early ([!589](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/589))
- Add `Asset` to the asset storage related type names ([!598](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/598))
- (mock) Replace `wait_accept` function with `enter_room` and use it in tests ([!603](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/603))

### 📦 Dependencies

- (deps) Lock file maintenance ([!581](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/581))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.90.0 ([!582](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/582))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.18.5 ([!584](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/584))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.18.5 ([!583](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/583))
- (deps) Update rust crate config to v0.15.17 ([!591](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/591))
- (deps) Update rust crates tungstenite and tokio-tungstenite to 0.28.0 ([!590](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/590))
- (deps) Update rust crate serde to v1.0.227 ([!596](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/596))
- (deps) Lock file maintenance ([!608](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/608))
- (deps) Update rust crate livekit to v0.7.19 ([!609](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/609))

### ⚙ Miscellaneous

- Remove old scripts that are replaced by the DUI ([!598](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/598))
- Remove old values from the example config ([!598](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/598))

### Ci

- Use fixed container tags ([!599](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/599))
- (renovate) Add group rule for livekit ([!609](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/609))

### Test

- (whiteboard) Add integration tests for whiteboard module ([!569](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/569))
- (mock) Don't allow to overwrite RoomParameters in RoomBuilder ([!598](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/598))

## [0.0.7] - 2025-09-19

[0.0.7]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.6...v0.0.7

### 🐛 Bug fixes

- (types) Remove unnecessary dependency ([!571](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/571))
- Remove self referencing dependency ([!571](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/571))

### 🔨 Refactor

- (test) Move tests to moderation module ([!571](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/571))

### 📦 Dependencies

- (deps) Update pre-commit hook daveshanley/vacuum to v0.18.3 ([!572](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/572))
- (deps) Update rust crate config to v0.15.16 ([!565](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/565))
- (deps) Update rust crate serde to v1.0.224 ([!568](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/568))
- (deps) Update rust crate serde to v1.0.225 ([!573](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/573))
- (deps) Update rust crate serde_path_to_error to v0.1.20 ([!566](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/566))

### ⚙ Miscellaneous

- (logging) Optional loopbacks are no errors ([!548](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/548))
- Use glob pattern for modules ([!571](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/571))
- Mark unpublished crates with `publish = false` ([!571](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/571))

### Ci

- Run livekit test in ci ([!548](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/548), [#88](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/88))

### Test

- Optionally use provided livekit server ([!548](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/548))
- (client) Add serialization tests for meeting notes command & event ([!570](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/570))

## [0.0.6] - 2025-09-15

[0.0.6]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.5...v0.0.6

### 🚀 New features

- (chat) Implement server side message search ([!469](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/469))
- (moderation) Change display name ([!476](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/476))
- (client) Add core command to client library ([!486](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/486))
- (types) Add `from_u128` impl to `DeviceId` ([!489](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/489))
- (moderation) Implement enable/disable waiting room ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- (moderation) Implement sent to waiting room ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- (moderation) Send waiting room `JoinInfo` to moderators ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- (raise-hands) Raise hands ([!485](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/485))
- (dui) Add timer plugin ([!503](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/503))
- (signaling) Add participant state to JoinInfo ([!494](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/494))
- Allow SignalingModules to add participant data ([!494](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/494))
- (timer) Add ready_state of peers to join success ([!494](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/494))
- Log if a module was skipped during initialization ([!507](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/507))
- (livekit) Add force mute internal command ([!490](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/490))
- (automod) Automod ([!490](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/490))
- Add call-in client kind ([!514](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/514))
- (room) Include module data about participants when switching breakout rooms ([!504](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/504), [#134](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/134))
- Rename peer data related fields ([!504](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/504))
- (subroom-audio) Add SubroomAudio Module ([!510](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/510))
- (dui) Add more default clients parameter ([!526](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/526))
- (dui) Enable all modules in default room parameter ([!526](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/526))
- (dui) About popup ([!526](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/526))
- (dui) Report more error details while connecting to roomserver ([!526](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/526))
- (dui) Use roomserver api token as default ([!526](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/526))
- (dui) Use all available editor width ([!526](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/526))
- (dui) Add event details and shared folder information to room parameter ([!526](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/526))
- (dui) Allow to suspend and resume websocket receiving ([!513](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/513))
- (dui) Add SpamAmountPlugin for spamming ([!513](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/513))
- (room) Send ParticipantDisconnected event when connection is dropped ([!513](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/513))
- (moderation) Add ban/unban commands ([!509](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/509))
- (room) Add timestamp to `SignalingEvent` ([!527](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/527))
- (room) Room time limit ([!530](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/530))
- (moderation) Allow moderators to change participant roles ([!537](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/537))
- (room) Add signaling event for commands sent from the waiting room ([!525](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/525))
- (ping) Respond to pings from the waiting room ([!525](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/525))
- (types) Introduce participation kind ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- (core) Add peer event data when participant joins ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519), [#118](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/118))
- (raise-hands) Use PeerEvent and PeerData instead of custom map ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- (core) Include participant information in JoinedWaitingRoom event ([!550](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/550))
- (storage) Add functions to upload a file in chunks ([!549](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/549))
- (meeting-notes) Meeting notes ([!549](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/549))
- (moderation) Notify moderators about accepted participants ([!531](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/531))
- (dui) Waiting room plugin ([!531](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/531))
- (dui) Use transaction IDs in plugins ([!531](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/531))

### 🐛 Bug fixes

- (chat) Prevent sending private messages to unknown participants ([!470](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/470))
- Tag core command for serialization ([!486](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/486))
- Prevent participants in waiting room from receiving broadcast events ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- (client) Use `payload` instead of `content` ([!503](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/503))
- Don't flatten Participant::module_data ([!494](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/494))
- Don't error if directory already exists ([!507](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/507))
- Send events and module messages in the correct order ([!490](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/490))
- (livekit) Send the correct identifier when muting a participant ([!512](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/512))
- Register raised hands signaling module ([!526](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/526))
- (room) Don't sleep 1 sec if connection becomes unresponsive ([!513](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/513), [#132](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/132))
- (livekit) Ensure muting works across breakout rooms ([!518](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/518))
- (moderation) Only notify participants that were unmuted when muting ([!518](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/518))
- (moderation) Enable waiting room when kicking participant ([!520](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/520))
- (moderation) Prevent room owner from getting kicked ([!520](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/520))
- (moderation) Regular users can enable/disable the waiting room ([!528](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/528))
- (dui) Widgets have duplicate ids when the same message is sent multiple times ([!529](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/529))
- (room) Tests do not compile when testing single module ([!530](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/530))
- (room) Waiting participants don't receive errors for core commands ([!525](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/525))
- (room) Core and breakout module features are missing in `JoinSuccess` ([!536](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/536))
- (livekit) Moderators can't screenshare when default screen share permission is false ([!543](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/543))
- (timer) Remove unnecessary serde tag ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- (raise-hands) Only reset raised hand if last connection is closed ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- (ci) Add trivy ignore file for trixie ([!551](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/551))
- (ci) Use nightly cargo for `cargo-fmt` in pre-commit hook ([!558](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/558))
- (dui) Don't dive into a busy loop when disconnecting ([!531](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/531))

### 📚 Documentation

- Fix mixed up field descriptions for `ModuleJoinData` and `ModuleSwitchData` ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))

### 🔨 Refactor

- (breakout) Move breakout module types to roomserver-types crate ([!489](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/489))
- (room) Deduplicate event serialization ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- (meeting-report) Remove `ParticipantKind` from `ReportParticipant` ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- (room) Store `ClientKind` and `Role` in `WaitingParticipant` ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- Use `ClientKind` in `ParticipantState` ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- Don't use RawJson ([!507](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/507))
- Remove async from functions where not necessary ([!515](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/515))
- (livekit) Rename `ForceMute` command to `Mute` ([!512](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/512))
- Remove result callback from internal command ([!512](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/512))
- Move microphone restriction commands to moderation module ([!512](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/512))
- Move mute command to moderation module ([!512](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/512))
- (room) Move core functions to core ([!513](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/513))
- Remove unnecessary mutex ([!513](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/513))
- (moderation) Add missing error variant doc comments ([!520](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/520))
- (justfile) Determine current version in functions ([!532](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/532))
- (room) Replace `IdleTimeout` with a generic `Timeout` ([!530](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/530))
- (room) Simplify `RoomTask::execute_signaling_module_command` ([!525](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/525))
- (room) Do not attempt to send error messages when no connection exist ([!525](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/525))
- (room) Ensure disconnected participants can't sent commands ([!525](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/525))
- (echo) Remove obsolete code from echo module ([!525](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/525))
- Introduce participant_id_from_uuid function ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- Add breakout join data methods ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- Don't panic on unserializable type ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- (livekit) Remove livekit client from `LivekitConnection` ([!549](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/549))
- Replace `FsStorage` with `MemoryFileStorage` ([!557](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/557))
- Implement `Copy` for `DisconnectReason` ([!531](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/531))

### 📦 Dependencies

- (deps) Lock file maintenance ([!472](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/472), [!493](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/493), [!506](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/506), [!524](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/524), [!545](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/545), [!564](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/564))
- (deps) Update egui to v0.32.3 ([!555](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/555))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.89.0 ([!468](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/468))
- (deps) Update opentalk ([!497](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/497))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.17.13 ([!559](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/559))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.18.4 ([!487](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/487))
- (deps) Update pre-commit hook fsfe/reuse-tool to v5.1.1 ([!544](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/544))
- (deps) Update pre-commit hook pre-commit/pre-commit-hooks to v6 ([!471](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/471))
- (deps) Update rust crate anyhow to v1.0.99 ([!479](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/479))
- (deps) Update rust crate async-trait to v0.1.89 ([!488](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/488))
- (deps) Update rust crate axum-prometheus to 0.9.0 ([!484](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/484))
- (deps) Update rust crate chrono to v0.4.42 ([!546](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/546))
- (deps) Update rust crate clap to v4.5.47 ([!533](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/533))
- (deps) Update rust crate config to v0.15.14 ([!482](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/482))
- (deps) Update rust crate insta to v1.43.2 ([!542](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/542))
- (deps) Update rust crate livekit to v0.7.18 ([!534](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/534))
- (deps) Update rust crate livekit-api to v0.4.6 ([!535](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/535))
- (deps) Update rust crate log to v0.4.28 ([!539](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/539))
- (deps) Update rust crate opentalk-types-api-v1 to v0.40.1 ([!511](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/511))
- (deps) Update rust crate opentalk-types-common to v0.36.1 ([!483](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/483))
- (deps) Update rust crate reqwest to v0.12.23 ([!480](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/480))
- (deps) Update rust crate serde_json to v1.0.143 ([!496](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/496))
- (deps) Update rust crate thiserror to v2.0.16 ([!498](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/498))
- (deps) Update rust crate url to v2.5.6 ([!502](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/502))

### ⚙ Miscellaneous

- Register missing modules ([!473](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/473))
- Adapt implementation for web compatibility ([!422](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/422))
- Rename ping module to echo for naming compatibility ([!495](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/495))
- Remove unused echo commands ([!495](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/495))
- Add insta files to git ignore ([!507](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/507))
- Wrap comments with rustfmt ([!538](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/538))
- (justfile) Check if yq is installed before running `tag-release` action ([!552](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/552))

### Ci

- Update debian based image trixie ([!475](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/475))
- Update debian based ci images to trixie ([!501](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/501))
- (nextest) Run all tests by default ([!553](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/553))
- Use nextest and llvm-cov for test coverage and report ([!553](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/553))
- Run tests on main ([!553](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/553))
- (just) Add scripts to get test coverage ([!553](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/553))

### Test

- (chat) Add test for breakout room history access permissions ([!469](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/469))
- (chat) Error when sending private messages to unknown participants ([!470](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/470))
- (types) Test serialization of `Participant` ([!489](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/489))
- Add tests that ensure waiting participants don't receive events ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- (moderation) Add integration tests for enable/disable waiting room ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- (moderation) Add integrations tests for send to waiting room ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- (raise-hands) Add integrations tests for raise hands ([!485](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/485))
- (timer) Introduce `start_timer` helper function ([!494](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/494))
- (types) Switch to insta ([!507](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/507))
- Move common livekit functions to separate crate ([!490](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/490))
- (automod) Add integration tests for automod module ([!490](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/490))
- (room) Add join info and switch into to MockModule ([!504](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/504))
- Integration test for switch and start breakout rooms ([!504](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/504))
- Use uniform test user names ([!526](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/526))
- (moderation) Verify livekit behavior when testing mute ([!518](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/518))
- (mock) Participants orderly disconnect in integration tests ([!547](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/547))
- Don't include connection_id as it's random and complicates testing ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- (mock) Verify that participant switched room ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- (raise-hands) Ensure state is reset when changing rooms ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- Allow configuring `receive_event` timeout ([!549](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/549))
- (meeting-notes) Add meeting notes tests ([!549](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/549))

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
