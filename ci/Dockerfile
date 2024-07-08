# SPDX-License-Identifier: EUPL-1.2
# SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
# SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates

WORKDIR /opentalk-roomserver

COPY target/release/opentalk-roomserver .
COPY example/config.toml config.toml

EXPOSE 11333
ENTRYPOINT [ "./opentalk-roomserver" ]
