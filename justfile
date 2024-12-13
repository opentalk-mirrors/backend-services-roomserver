# SPDX-License-Identifier: EUPL-1.2
# SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
# SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
#
# This file can be used with the [`just`](https://just.systems) tool.

[no-exit-message]
_check_cargo_set_version:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! cargo set-version --help &>/dev/null; then
        echo 'cargo set-version is not available, you can install it with `cargo install cargo-edit`' >&2
        exit 1
    fi

# Prepare a release
prepare-release VERSION: _check_cargo_set_version
    # Set the version number for all packages in the workspace
    cargo set-version --workspace {{ VERSION }}
    # Regenerate the lockfile
    cargo check

# Update the version in the OpenAPI spec
update-frontend-api:
    # Update OpenAPI specification (which contains the version number)
    cargo run -- openapi dump > api/docs/openapi.yml
