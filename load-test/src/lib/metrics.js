// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

export function shared_tags(testName, startTime) {
  return {
    vu: `${__VU}`,
    iteration: `${__ITER}`,
    test_name: `${testName}-${startTime.toTimeString()}`,
  };
}
