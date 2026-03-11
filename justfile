# SPDX-License-Identifier: EUPL-1.2
# SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
# SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>
#
# This file can be used with the [`just`](https://just.systems) tool.

set quiet := true

[no-exit-message]
_check_cargo_set_version:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! cargo set-version --help &>/dev/null; then
        echo 'cargo set-version is not available, you can install it with `cargo install cargo-edit`' >&2
        exit 1
    fi

[no-exit-message]
_check_ci_doc_updater:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v opentalk-ci-doc-updater > /dev/null; then
        echo 'opentalk-ci-doc-updater is not available, you can install it with' >&2
        echo '`cargo install --git ssh://git@git.opentalk.dev:222/opentalk/tools/opentalk-ci-doc-updater.git`' >&2
        exit 1
    fi

[no-exit-message]
_check_cargo_depgraph:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! cargo --list | grep "depgraph" > /dev/null; then
        echo 'cargo-depgraph is not available, please install it with`' >&2
        echo '`cargo install cargo-depgraph`' >&2
        exit 1
    fi

[no-exit-message]
_check_dot:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v dot > /dev/null; then
        echo 'dot is not available, please install Graphviz.`' >&2
        exit 1
    fi

[no-exit-message]
_check_opentalk_git_cliff:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! opentalk-git-cliff --help &>/dev/null; then
        echo 'opentalk-git-cliff is not available, you can install it with `cargo install --git ssh://git@git.opentalk.dev:222/opentalk/tools/check-changelog.git`' >&2
        exit 1
    fi

[no-exit-message]
_check_yq:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! yq --help &>/dev/null; then
        echo 'yq is not available, see https://github.com/mikefarah/yq' >&2
        exit 1
    fi

# Prepare a release
prepare-release VERSION: (set-version VERSION) update-openapi (update-changelog VERSION)

# Sets the version in the Cargo.toml and updates the Cargo.lock
set-version VERSION: _check_cargo_set_version _check_yq
    # Set the version number for all packages in the workspace
    cargo set-version --workspace {{ VERSION }}
    # Regenerate the lockfile
    cargo check
    # update the frontend api
    yq '.info.version = "{{ VERSION }}"' -i api/docs/openapi.yml

# Update the version in the OpenAPI spec
update-openapi:
    # Update OpenAPI specification (which contains the version number)
    cargo run -- openapi dump > api/docs/openapi.yml
    # Trim whitespace
    sed -i 's/[[:space:]]*$//' api/docs/openapi.yml
    # Add trailing new line (removed by previous command)
    echo '' >> api/docs/openapi.yml

# Update the changelog
update-changelog VERSION: _check_opentalk_git_cliff
    #!/usr/bin/env bash

    if [ -z "$GITLAB_TOKEN" ] && [ -f "$HOME/.gitlab_token" ]; then
        GITLAB_TOKEN=$(cat $HOME/.gitlab_token)
    fi

    # Update Changelog
    GITLAB_TOKEN=$GITLAB_TOKEN \
    GITLAB_API_URL=https://git.opentalk.dev/api/v4 \
    GITLAB_REPO=opentalk/backend/services/roomserver \
    opentalk-git-cliff \
        --unreleased \
        --tag "v{{ VERSION }}" \
        --prepend CHANGELOG.md

# Create the release commit
commit-release: _check_yq
    #!/usr/bin/env bash
    current_version=$(cat Cargo.toml | yq -ptoml .workspace.package.version)
    git commit -a -m "chore(release): prepare release $current_version"
    git log HEAD^..HEAD

# Create the release tag
tag-release: _check_yq
    #!/usr/bin/env bash
    current_version=$(cat Cargo.toml | yq -ptoml .workspace.package.version)
    git tag -s -m "v$current_version" "v$current_version"
    git show --no-patch "v$current_version"

# Update generated or derived parts of the documentation
update-docs: _check_ci_doc_updater
    cargo build --release
    ./ci/docs-update-generated-parts.sh

run-dui *ARGS:
    RUST_LOG=opentalk=debug cargo run -p opentalk-roomserver-dui -- {{ ARGS }}

generate-deps-graph: _check_cargo_depgraph _check_dot
    #!/usr/bin/env bash
    set -euo pipefail
    OUT_PATH="target/dep-graph.png"
    cargo depgraph --workspace-only --all-deps | dot -Tpng > $OUT_PATH
    echo "Created dependency graph at $OUT_PATH"

test-coverage:
    cargo llvm-cov nextest --lcov --output-path lcov.info

install-latest-typst-packages:
    #!/usr/bin/env bash
    # pull the image with the typst packages
    docker pull git.opentalk.dev:5050/opentalk/backend/containers/typst-plugins:scratch-dev
    # create a new container
    # sh is necessary because creating containers without entry point is not possible
    CONTAINER_NAME=typst-plugins
    docker create --name "$CONTAINER_NAME" git.opentalk.dev:5050/opentalk/backend/containers/typst-plugins:scratch-dev sh >/dev/null 2>&1 || true
    # create the typst package directory
    TYPST_DIR=${XDG_CACHE_HOME:-$HOME/.cache}/typst/packages/preview/
    mkdir -p "$TYPST_DIR"
    # remove existing package
    rm -r "$TYPST_DIR/linguify" >/dev/null 2>&1 || true
    # copy the typst packages from the container
    docker cp typst-plugins:/usr/share/typst/packages/preview/linguify/ "$TYPST_DIR"
    # delete the container
    docker container rm "$CONTAINER_NAME"
