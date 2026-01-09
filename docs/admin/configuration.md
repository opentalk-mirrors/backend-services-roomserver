---
title: Configuration
---

# Configuring {{ product_name }} RoomServer

The {{ product_name }} RoomServer can be configured using environment variables and a configuration file.
Environment variables take precedence over the configuration file.

## Environment variables

Settings in the configuration file can be overwritten by environment variables,
nested fields are separated by two underscores `__`. The pattern looks like
this:

```sh
OT_ROOMSERVER_<field>__<nested-field>…
```

## Sections in the configuration file

- [Monitoring](observability/monitoring.md)
- [Metrics](observability/metrics.md)
- [Tracing](observability/tracing.md)
- [Defaults](defaults.md)
- [Conference](conference.md)

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
public_url = "http://localhost:11333"

# The api keys for internal service endpoints
#
# The roomserver can have multiple api keys configured. An api key can be configured as string ("<key_id>:<key_secret>")
# or as key/value pair ({id = "<key_id>", secret = "<key_secret>"}).
api_keys = [{ id = "roomserver", secret = "secret" }]

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
# IP addresses allowed to access the metrics endpoint
#allowlist = []

# Allow access from localhost
#allowlist = ["127.0.0.0/24", "::ffff:0:0/96"]

#[tracing]
# Tracing settings

# Tracing is optional and disabled by default
#otlp_tracing_endpoint = "http://localhost:4317"

# Conference settings are optional
#[conference]
# Conference settings

# Used to derive participant ids from device secrets.
# A random salt is generated at startup when not set. This value must be kept secret.
# signaling_salt = "random string at least 24 characters long"

# The duration in seconds after which a room without participants is closed.
# room_idle_timeout = "60"

# [defaults]
# When true, participants can't share their screens unless permission is granted by a moderator.
# Moderators can always share their screen.
# screen_share_requires_permission = true

#[reports.typst]
# The location where typst looks for packages.
#packages_path = "/usr/share/typst/packages"
```

<!-- end:fromfile:example/roomserver.toml.md -->
