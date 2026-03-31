// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
/**
 * Spike Storage Quota Test
 * Creates multiple rooms by ramping up virtual users, each joining their own room.
 * While the rooms are active, a separate scenario sends HTTP POST requests to update
 * a user's storage quota. All VUs share the same user ID, so the corresponding `StorageQuotaChanged`
 * core event is sent to each room.
 *  A single measuring user joins its own room and continuously sends echo requests to track the
 * roomserver's responsiveness during the spike.
 * The goal is to observe how storage quota updates affect echo round-trip times.
 *
 * Metrics:
 * - spike_storage_quota_echo_rtt: gauge of per-request round-trip times in milliseconds
 *   of the measuring user.
 */
import { sleep } from 'k6';
import exec from 'k6/execution';
import http from 'k6/http';
import { Gauge } from 'k6/metrics';

import { generateJWT } from '../lib/auth.js';
import { ClientBuilder } from '../lib/client.js';
import { getEnv, getRequiredEnv } from '../lib/environment.js';
import { shared_tags } from '../lib/metrics.js';

// Test metadata
const TEST_NAME = 'spike_storage_quota';
const TEST_START_TIME = new Date();

// Test configuration
const BASE_URL = getRequiredEnv('BASE_URL');
const MEASURE_DURATION = getEnv('SPIKE_STORAGE_QUOTA_MEASURE_DURATION', 70); // seconds
const ECHO_INTERVAL = getEnv('SPIKE_STORAGE_QUOTA_ECHO_INTERVAL', 1); // seconds

// Custom metrics
const ECHO_RTT = new Gauge('spike_storage_quota_echo_rtt');

export const options = {
  scenarios: {
    rooms: {
      exec: 'createAndHoldRoom',
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '10s', target: 10 }, // Ramp up to 10 VUs over 10 seconds
        { duration: '50s', target: 10 }, // Keep at 10 VUs for 30 seconds
        { duration: '10s', target: 0 }, // Ramp down to 0 VUs over 10 seconds
      ],
    },
    request: {
      exec: 'post_storage_quota',
      executor: 'constant-vus',
      vus: 10,
      startTime: '20s', // Start sending requests after 20 seconds to allow rooms to be created first
      duration: '30s',
    },
    measure: {
      exec: 'measure',
      executor: 'constant-vus',
      vus: 1,
      duration: `${MEASURE_DURATION}s`,
    },
  },
};

export async function createAndHoldRoom() {
  let client;
  try {
    const roomId = crypto.randomUUID();
    client = await new ClientBuilder().connect(BASE_URL, roomId);
    console.info(`VU ${__VU} connected to room ${roomId}`);

    // Wait until k6 stops the VU (and cancels the sleep timer)
    sleep(31557600);

    console.info(`VU ${__VU} completed test duration. Disconnecting.`);
  } catch (err) {
    exec.test.abort(`VU ${__VU} encountered an error: ${err}`);
  } finally {
    client?.disconnect();
  }
}

export async function post_storage_quota() {
  const userId = '00000000-0000-0000-0000-0000000a11ce';
  const body = JSON.stringify({
    total: 2,
    used: 1,
  });
  const jwt = generateJWT();
  const params = {
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${jwt}`,
    },
  };
  const response = http.post(`${BASE_URL}/v1/user/${userId}/storage_quota`, body, params);
  if (response.status < 200 || response.status > 299) {
    console.error(
      `VU ${__VU} received unexpected status ${response.status} for storage quota request: ${response.body}`
    );
  }
}

export async function measure() {
  let transactionId = 0;
  const roomId = crypto.randomUUID();
  // Use a different user ID for the measuring client to ensure it does not receive the StorageQuotaChanged event
  const userId = '00000000-0000-0000-0000-000000ea500e';
  const client = await new ClientBuilder().connect(BASE_URL, roomId, userId);
  console.info(`VU ${__VU} connected for measuring`);

  try {
    const measureDurationMs = MEASURE_DURATION * 1000;
    while (exec.instance.currentTestRunDuration < measureDurationMs) {
      const startTime = exec.instance.currentTestRunDuration;
      await client.sendEcho(transactionId++);

      // Record metrics
      const rtt = exec.instance.currentTestRunDuration - startTime;
      ECHO_RTT.add(rtt, shared_tags(TEST_NAME, TEST_START_TIME));

      // Wait between echo requests
      sleep(ECHO_INTERVAL);
    }
    console.info(`VU ${__VU} completed measuring. Disconnecting.`);
  } catch (err) {
    exec.test.abort(`VU ${__VU} encountered an error during measuring: ${err}`);
  } finally {
    client.disconnect();
  }
}
