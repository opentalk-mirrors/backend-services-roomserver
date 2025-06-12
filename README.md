# OpenTalk RoomServer

This project contains the crates that implement the OpenTalk RoomServer.

- [OpenTalk RoomServer](opentalk-roomserver/src/main.rs) – The crate which produces the actual RoomServer executable
- [OpenTalk RoomServer Common](opentalk-roomserver-common/src/lib.rs) – Types that are shared between the OpenTalk RoomServer and Signaling Modules
- [OpenTalk RoomServer Client](./opentalk-roomserver-client/src/lib.rs) – HTTP-Client which can be used to communicate with the OpenTalk RoomServer using the _OpenTalk RoomServer Web API_
- [OpenTalk RoomServer Types](./opentalk-roomserver-types/src/lib.rs) – Types that are send over the wire. These types are used in the HTTP-API and used in both the RoomServer-Client and RoomServer-Web-API.
- [OpenTalk RoomServer Web API](./opentalk-roomserver-web-api/src/lib.rs) – HTTP-API endpoints written with Axum. This crate defines the API but does not implement any logic. All requests are forwarded to trait implementations.
- [OpenTalk RoomServer Signaling](./opentalk-roomserver-signaling/src/lib.rs) – This crate contains types required by signaling modules. Most prominently the `SignalingModule` trait.

## Developer UI

The RoomServer DUI is used to debug the signaling interface.

You can execute it using:

```bash
just run-dui
```

## Documentation

Documentation for the RoomServer is build in [the unified documentation project](https://git.opentalk.dev/opentalk/backend/docs).
The content is copied from `<roomserver-project-root>/docs` and placed in the documentation project automatically.

### Copy files outside of `docs/`

The documentation contains verbatim copies of files outside the `docs/` folder
(e.g. `examples/roomserver.toml`). The [`ci-docs-updater`](https://git.opentalk.dev/opentalk/tools/opentalk-ci-doc-updater) is used to ensure that the documentation stays in sync with the original files.

You can update the documentation running:

```bash
just update-docs
```
