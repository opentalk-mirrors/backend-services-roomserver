# OpenTalk RoomServer

This project contains the crates that implement the OpenTalk RoomServer.

- [OpenTalk RoomServer](opentalk-roomserver/src/main.rs) – The crate which produces the actual RoomServer executable
- [OpenTalk RoomServer Client](./opentalk-roomserver-client/src/lib.rs) – HTTP-Client which can be used to communicate with the OpenTalk RoomServer using the _OpenTalk RoomServer Web API_
- [OpenTalk RoomServer Types](./opentalk-roomserver-types/src/lib.rs) – HTTP-Client which can be used to communicate with the OpenTalk RoomServer using the _OpenTalk RoomServer Web API_
- [OpenTalk RoomServer Web API](./opentalk-roomserver-web-api/src/lib.rs) – HTTP-API endpoints written with Axum. This crate defines the API but does not implement any logic. All requests are forwarded to trait implementations.
