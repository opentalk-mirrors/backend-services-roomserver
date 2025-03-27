#!/usr/bin/env bash

# SPDX-License-Identifier: EUPL-1.2
# SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

set -e -o pipefail

if ! command -v websocat > /dev/null; then
    echo "no websocat installation found, install with 'cargo install --features=ssl websocat'" >&2
    exit 1
fi

if ! command -v jq > /dev/null; then
    echo "no jq installation found" >&2
    exit 1
fi

script_dir=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

HOST=${HOST:-"[::]:11311"}
ROOM_ID=${ROOM_ID:-"00000000-0000-0000-0001-000000000001"}
API_TOKEN=${API_TOKEN:-"secret"}

# Put a new room

echo "Put room '$ROOM_ID'"
curl -i -XPUT "http://$HOST/v1/rooms/$ROOM_ID" \
    -H "Content-type: application/json" \
    -H "Authorization: bearer $API_TOKEN" \
    --data "@$script_dir/create_room.json"

# Request a token

echo "Request access token for room '$ROOM_ID'"
token_response=$( curl -XPOST "http://$HOST/v1/rooms/$ROOM_ID/token" \
    -H "Content-type: application/json" \
    -H "Authorization: bearer $API_TOKEN" \
    --data "@$script_dir/post_token.json" )

room_token=$( echo "$token_response" | jq -r ".token" )

echo "Received room access token: '$room_token'"

websocat \
    "ws://$HOST/signaling/$room_token"
