# OpenTalk RoomServer

This project contains the crates that implement the OpenTalk RoomServer.

- [OpenTalk RoomServer](opentalk-roomserver/src/main.rs) – The crate which produces the actual RoomServer executable
- [OpenTalk RoomServer Client](./opentalk-roomserver-client/src/lib.rs) – HTTP-Client which can be used to communicate with the OpenTalk RoomServer using the _OpenTalk RoomServer Web API_
- [OpenTalk RoomServer Types](./opentalk-roomserver-types/src/lib.rs) – HTTP-Client which can be used to communicate with the OpenTalk RoomServer using the _OpenTalk RoomServer Web API_
- [OpenTalk RoomServer Web API](./opentalk-roomserver-web-api/src/lib.rs) – HTTP-API endpoints written with Axum. This crate defines the API but does not implement any logic. All requests are forwarded to trait implementations.

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
(e.g. `examples/config.toml`). The [`ci-docs-updater`](https://git.opentalk.dev/opentalk/tools/opentalk-ci-doc-updater) is used to ensure that the documentation stays in sync with the original files.

You can update the documentation running:

```bash
just update-docs
```

### Style of `examples/config.toml`

The `config.toml` is exposed in our documentation. It should therefore follow some styling rules to make it easier to use.

#### Example

```toml
[table-1]
# Table 1 is a configuration section for the RoomServer

# option-a is optional and defaults to 42
#option-a = 42
```

#### Rules

##### Optional values are commented out with their default value

```toml
# option-a is optional and defaults to 42
#option-a = 42
```

##### tables are not commented out

- this makes it easier to change settings, since only a single line needs to be uncommented
- it also highlights the tables if you have syntax highlighting for toml files
- do:

    ```toml
    [table-1]
    ```

- don't:

    ```toml
    #[table-1]
    ```

##### configuration values don't have a space after the hashtag if commented out

- do:

    ```toml
    #option-a = 42
    ```

- don't:

    ```toml
    # option-a = 42
    ```

##### comments are below the table

- do:

    ```toml
    [table-1]
    # Table 1 configures the RoomServer
    ```

- don't:

    ```toml
    # Table 1 configures the RoomServer
    [table-1]
    ```
