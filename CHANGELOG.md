# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
