// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
import crypto from 'k6/crypto';
import encoding from 'k6/encoding';
import exec from 'k6/execution';
import http from 'k6/http';

import { getEnv } from './environment.js';

// JWT secret key (this should match your roomserver configuration)
const JWT_SECRET = getEnv('JWT_SECRET', 'secret');
const JWT_KID = getEnv('JWT_KID', 'roomserver');

/**
 * Generate a JWT token for authentication
 */
export function generateJWT() {
  const now = Math.floor(Date.now() / 1000);

  const header = {
    alg: 'HS256',
    typ: 'JWT',
    nonce: '12345',
    kid: JWT_KID,
  };

  const payload = {
    sub: '1234567890',
    iat: now,
    exp: now + 3600, // expires in 1 hour
  };

  const encodedHeader = encoding.b64encode(JSON.stringify(header), 'rawurl');
  const encodedPayload = encoding.b64encode(JSON.stringify(payload), 'rawurl');
  const message = `${encodedHeader}.${encodedPayload}`;

  const signature = crypto.hmac('sha256', JWT_SECRET, message, 'base64rawurl');

  return `${message}.${signature}`;
}

/**
 * Request a room token from the API
 */
export function requestRoomToken(baseUrl, roomId, tokenRequest) {
  const jwt = generateJWT();

  const payload = JSON.stringify(tokenRequest);

  const params = {
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${jwt}`,
    },
  };

  const response = http.post(`${baseUrl}/v1/rooms/${roomId}/token`, payload, params);

  try {
    return JSON.parse(response.body);
  } catch (error) {
    console.log(`Token request failed: ${response.body}`);
    exec.test.abort('token request failed');
    throw error;
  }
}

/**
 * Get WebSocket signaling URL from access info
 */
export function getSignalingUrl(accessInfo) {
  const public_url = accessInfo.public_url.replace(/^http:\/\//i, 'ws://');
  return `${public_url}v1/signaling/${accessInfo.token}`;
}
