// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
import exec from 'k6/execution';
import { WebSocket } from 'k6/experimental/websockets';
import { setTimeout, clearTimeout } from 'k6/timers';

import { requestRoomToken, getSignalingUrl } from './auth.js';
import { sendCommand } from './messaging.js';
import { createClientParameter, createRoomParameter } from './parameters.js';

export class ClientBuilder {
  /**
   * Open a WebSocket connection to the RoomServer and join the specified room
   * @param {string} baseUrl - Base URL of the RoomServer
   * @param {string} roomId - The ID of the room to join
   * @returns {Promise<Client>} The connected client
   */
  async connect(baseUrl, roomId) {
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

    // Connect websocket
    const ws = new WebSocket(signalingUrl);

    // Wait until we receive the join_success message
    await new Promise((resolve, reject) => {
      // Fail if we don't receive the join_success message in time
      const timerId = setTimeout(() => {
        const msg = 'Timeout waiting for join_success message';
        reject(new Error(msg));
        exec.test.abort(msg);
      }, 1_000);

      ws.onmessage = (event) => {
        const message = JSON.parse(event.data);
        if (message.payload.message === 'join_success') {
          clearTimeout(timerId);
          resolve();
        }
      };
    });

    return new Client(ws);
  }
}

class Event {
  constructor(namespace, transactionId, resolve) {
    this.namespace = namespace;
    this.transactionId = transactionId;
    this.resolve = resolve;
  }

  /**
   * Check if the event matches this event's namespace and transaction ID
   * @param {any} event - The event to compare against
   * @returns {boolean} - True when the event matches this event's namespace and transaction ID
   */
  matches(event) {
    return event && this.namespace === event.namespace && this.transactionId === event.transaction_id;
  }
}

export class Client {
  constructor(ws) {
    this.ws = ws;
    this.events = [];

    ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      const matchingEvent = this.events.find((e) => e.matches(message));
      if (matchingEvent) {
        matchingEvent.resolve(message);
        this.events = this.events.filter((e) => e !== matchingEvent);
      } else {
        // This happens when broadcasts are triggered by other clients
        console.debug(`Received unmatched message: ${event.data}`);
      }
    };

    ws.onerror = (error) => {
      console.error(`Received error from WebSocket: ${JSON.stringify(error)}`);
    };
  }

  /**
   * Sends an arbitrary command to the RoomServer
   * @param {string} namespace - The namespace of the command
   * @param {any} payload - The content of the command
   * @param {number} transactionId - The transaction ID of the command
   * @returns {Promise<any>} The corresponding event send by the RoomServer
   */
  sendCommand(namespace, payload, transactionId) {
    const self = this;
    const promise = new Promise((resolve, _reject) => {
      self.events.push(new Event(namespace, transactionId, resolve));
    });
    sendCommand(this.ws, namespace, payload, transactionId);
    return promise;
  }

  /**
   * Sends a echo ping command
   * @param {number} transactionId - The transaction ID of the command
   * @returns {Promise<any>} The corresponding event send by the RoomServer
   */
  sendEcho(transactionId) {
    return this.sendCommand('echo', { action: 'ping' }, transactionId);
  }

  /**
   * Disconnects the client, closing its WebSocket connection
   */
  disconnect() {
    this.ws.close();
  }
}
