// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Contains code for room management and signaling.
//!
//! The room state is managed by the [`task::RoomTask`], where each room has its own [`tokio::task`] with an instance of
//!  a [`RoomTask`](task::RoomTask). The [`RoomTasks`](task::RoomTask) have a channel interface that is exposed via the
//! [`handle::RoomTaskHandle`] through which the web api can send [`TaskMessages`](handle::TaskMessage) requests to each
//! individual room.
//!
//! The active rooms are created and tracked with the [`RoomTaskRegistry`](registry::RoomTaskRegistry). When a
//! [`task::RoomTask`] gets destroyed, it removes itself from the [`RoomTaskRegistry`](registry::RoomTaskRegistry)
//! through its drop implementation.

pub(crate) mod handle;
mod idle_timeout;
pub(crate) mod registry;
pub(crate) mod task;
