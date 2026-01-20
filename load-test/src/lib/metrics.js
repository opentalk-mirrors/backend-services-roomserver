// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

/**
 * @param {string} testName - Name of the test
 * @param {Date} startTime - Start time of the test, used to differentiate multiple runs
 * @returns {Object} Tags object for metrics
 */
export function shared_tags(testName, startTime) {
  return {
    vu: `${__VU}`,
    iteration: `${__ITER}`,
    test_name: `${testName}-${startTime.toTimeString()}`,
  };
}
