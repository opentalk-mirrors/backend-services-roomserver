---
title: Configuration
---

# Configuring OpenTalk-RoomServer

The RoomServer can be configured using environment variables and a configuration file.
Environment variables take precedence over the configuration file.

## Environment variables

Settings in the configuration file can be overwritten by environment variables,
nested fields are separated by two underscores `__`. The pattern looks like
this:

```sh
OT_ROOMSERVER_<field>__<nested-field>…
```

## Example configuration file

This file can be found in the source code distribution under `example/roomserver.toml`

<!-- begin:fromfile:example/roomserver.toml.md -->

```toml
[http]
# HTTP server settings

# The IPv4/IPv6 address to bind the HTTP server to.
# Binds to all interfaces by default:
#address = "0.0.0.0"

# All IPv6 interfaces:
#address = "::"

# Localhost only:
#address = "127.0.0.1"
#address = "::1"

# The publicly reachable URL of this server
# public_url = "http://localhost:11333"

# The api token for internal service endpoints
api_token = "secret"

# The port to bind the HTTP server to (defaults to 11333)
#port = 11333

# Disable the OpenAPI endpoint under `/docs/openapi.json` and the corresponding
# swagger endpoint under `/swagger`.
#disable_openapi = false

[monitoring]
# Monitoring settings

# Monitoring is optional and disabled by default
#addr = "0.0.0.0"
#port = 11411

[metrics]
# Metrics settings

# Metrics is optional and disabled by default
#port = 11412

#[tracing]
# Tracing settings

# Tracing is optional and disabled by default
#otlp_tracing_endpoint = "http://localhost:4317"

# LiveKit WebRTC SFU
[livekit]
public_url = "http://localhost:7880"
service_url = "http://localhost:7880"

api_key = "devkey"
api_secret = "secret"

# [defaults]
# When true, participants can't share their screens unless permission is granted by a moderator.
# Moderators can always share their screen.
# screen_share_requires_permission = true
```

<!-- end:fromfile:example/roomserver.toml.md -->
