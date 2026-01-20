// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
import { WebSocket } from 'k6/experimental/websockets';
import { Counter, Trend } from 'k6/metrics';

import { requestRoomToken, getSignalingUrl } from './auth.js';
import { sendCommand } from './messaging.js';
import { shared_tags } from './metrics.js';
import { createRoomParameter, createClientParameter } from './parameters.js';

// Custom metrics
export const echoResponsesCounter = new Counter('echo_responses');
export const echoRTT = new Trend('echo_rtt', true);

/**
 * Run an echo stress test session
 * @param {string} baseUrl - The base URL of the roomserver
 * @param {string} roomId - The room ID to join
 * @param {number} sessionDuration - Duration of the session in milliseconds
 * @param {string} testName - Name of the test for metrics tagging
 * @param {Date} testStartTime - Start time of the test for metrics tagging
 */
export function echo_load(baseUrl, roomId, sessionDuration, testName, testStartTime) {
  // Create parameters for this virtual user
  const roomParams = createRoomParameter();
  const clientParams = createClientParameter(__VU, __ITER);

  // Request room token
  const accessInfo = requestRoomToken(baseUrl, roomId, {
    room_parameters: roomParams,
    client_parameters: clientParams,
  });

  // Get WebSocket URL
  const signalingUrl = getSignalingUrl(accessInfo);

  let echoCount = 0;
  let waitingForResponse = false;
  let transactionId = 1;
  let isClosed = false;
  let echoSentTime = 0;

  console.log(`VU ${__VU}: Session duration: ${sessionDuration}ms`);

  const ws = new WebSocket(signalingUrl);

  // Close the WebSocket after the session duration
  const sessionTimeout = setTimeout(() => {
    console.log(`VU ${__VU}: Session duration reached (${sessionDuration}ms), closing connection`);
    isClosed = true;
    ws.close();
  }, sessionDuration);

  function sendEcho() {
    if (isClosed || waitingForResponse || ws.readyState !== 1) {
      return;
    }

    try {
      sendCommand(ws, 'echo', { action: 'ping' }, transactionId++);
      echoSentTime = Date.now();
      waitingForResponse = true;
    } catch (e) {
      console.error(`VU ${__VU}: Error sending echo: ${e.message || e}`);
      isClosed = true;
      ws.close();
    }
  }

  ws.addEventListener('open', () => {
    console.log(`VU ${__VU}: Connected, readyState=${ws.readyState}`);
  });

  ws.addEventListener('message', (event) => {
    if (isClosed) return;

    const message = JSON.parse(event.data);

    // Handle different message types
    if (message.namespace === 'core') {
      if (message.payload.message === 'join_success') {
        console.log(`VU ${__VU}: Joined room, starting echo loop`);
        sendEcho();
      }
    } else if (message.namespace === 'echo') {
      if (message.payload.message === 'pong') {
        const rtt = Date.now() - echoSentTime;
        echoRTT.add(rtt, shared_tags(testName, testStartTime));
        echoCount++;
        echoResponsesCounter.add(1, shared_tags(testName, testStartTime));
        waitingForResponse = false;

        // Send next echo command immediately
        sendEcho();
      }
    }
  });

  ws.addEventListener('close', () => {
    isClosed = true;
    clearTimeout(sessionTimeout);
    console.log(`VU ${__VU}: Disconnected. Total echo responses: ${echoCount}`);
  });

  ws.addEventListener('error', (e) => {
    isClosed = true;
    clearTimeout(sessionTimeout);
    console.error(`VU ${__VU}: WebSocket error: ${e.error}`);
    ws.close();
  });
}
