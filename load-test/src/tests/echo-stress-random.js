// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
/**
 * Echo Stress Test - Random Duration
 *
 * This test establishes WebSocket connections and continuously sends echo ping commands
 * to measure server responsiveness under load. Each virtual user has a random session
 * duration and sends echo requests as fast as the server can respond, tracking round-trip
 * times and response counts.
 *
 * This scenario starts all VUs immediately without a ramp-up phase.
 */
import { echo_load } from '../lib/echo-load.js';
import { getEnv, getRequiredEnv } from '../lib/environment.js';

// Test metadata
const TEST_START_TIME = new Date();
const TEST_NAME = 'Echo Stress Test - Random Duration';

// Test configuration
const BASE_URL = getRequiredEnv('BASE_URL');
const ROOM_ID = getEnv('ROOM_ID', '27c66df5-f6be-4d70-a167-abba2cf28a2a');
const MIN_SESSION_DURATION = 10000; // 10 seconds
const MAX_SESSION_DURATION = 60000; // 60 seconds

// Test scenario
export const options = {
  scenarios: {
    random_duration: {
      executor: 'constant-vus',
      vus: 10,
      duration: '3m',
    },
  },
};

export default function () {
  const sessionDuration =
    Math.floor(Math.random() * (MAX_SESSION_DURATION - MIN_SESSION_DURATION + 1)) + MIN_SESSION_DURATION;
  echo_load(BASE_URL, ROOM_ID, sessionDuration, TEST_NAME, TEST_START_TIME);
}
