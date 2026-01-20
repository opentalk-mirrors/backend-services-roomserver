// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
/**
 * Echo Stress Test - Fixed Duration
 *
 * This test establishes WebSocket connections and continuously sends echo ping commands
 * to measure server responsiveness under load. Each virtual user maintains a fixed session
 * duration and sends echo requests as fast as the server can respond, tracking round-trip
 * times and response counts.
 *
 * This scenario uses a gradual ramp-up to avoid connection bursts.
 */
import { echo_load } from '../lib/echo-load.js';
import { getEnv, getRequiredEnv } from '../lib/environment.js';

// Test metadata
const TEST_START_TIME = new Date();
const TEST_NAME = 'Echo Stress Test - Fixed Duration';

// Test configuration
const BASE_URL = getRequiredEnv('BASE_URL');
const ROOM_ID = getEnv('ROOM_ID', '27c66df5-f6be-4d70-a167-abba2cf28a2a');
const SESSION_DURATION = 30000; // 30 seconds

// Test scenario
export const options = {
  scenarios: {
    fixed_duration: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '30s', target: 10 }, // Ramp up to 10 VUs over 30s
        { duration: '2m', target: 10 }, // Stay at 10 VUs for 2m
        { duration: '30s', target: 0 }, // Ramp down to 0 VUs over 30s
      ],
      gracefulRampDown: '10s',
    },
  },
};

export default function () {
  echo_load(BASE_URL, ROOM_ID, SESSION_DURATION, TEST_NAME, TEST_START_TIME);
}
