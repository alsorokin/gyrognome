import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import test from 'node:test';

import {
  CLOCK_CONTROL,
  DEFAULT_CLOCK_START_MS,
  MOTTO_CHANGE_TEXT,
  OFFICIAL_ENDPOINTS,
  REPORT_FIELDS,
  SCENARIOS,
  assertCredentialFree,
  bridgeActionsForScenario,
  buildClockInitScript,
  compareTraceObservations,
  expectedObservationsFromBridgeEvents,
  isAllowedOfficialRequest,
  parseOptions,
  safeRequestObservation,
} from '../scripts/leaderboard-conformance.mjs';

test('requires disposable confirmation before Playwright can start', () => {
  const result = spawnSync(process.execPath, ['scripts/leaderboard-conformance.mjs'], {
    encoding: 'utf8',
  });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /--confirm-disposable/);
  assert.doesNotMatch(result.stderr, /playwright|passkey|cmd=/i);
});

test('redacts unsafe option values from harness errors', () => {
  const result = spawnSync(
    process.execPath,
    ['scripts/leaderboard-conformance.mjs', '--unrecognized=passkey'],
    { encoding: 'utf8' },
  );
  assert.equal(result.status, 1);
  assert.doesNotMatch(result.stderr, /passkey/i);
});

test('does not enable report submission without disposable confirmation', () => {
  assert.throws(() => parseOptions(['--submit']), /--confirm-disposable/);
});

test('rejects all managed-character input forms', () => {
  for (const option of ['--character-id=existing', '--managed-character', '--id=existing']) {
    assert.throws(
      () => parseOptions(['--confirm-disposable', option]),
      /managed-character input/,
    );
  }
});

test('allows only the observed official pages and leaderboard endpoint', () => {
  assert.ok(isAllowedOfficialRequest(OFFICIAL_ENDPOINTS.play));
  assert.ok(isAllowedOfficialRequest(OFFICIAL_ENDPOINTS.config));
  assert.ok(isAllowedOfficialRequest(OFFICIAL_ENDPOINTS.client));
  assert.ok(isAllowedOfficialRequest(`${OFFICIAL_ENDPOINTS.leaderboard}?cmd=create`));
  assert.ok(isAllowedOfficialRequest(`${OFFICIAL_ENDPOINTS.leaderboard}?cmd=b&t=s`));
  assert.equal(isAllowedOfficialRequest('https://example.invalid/play/'), false);
  assert.equal(isAllowedOfficialRequest('https://progressquest.com/unrelated.js'), false);
  assert.equal(isAllowedOfficialRequest('https://www.progressquest.com/cgi-bin/other.pl'), false);
  assert.equal(isAllowedOfficialRequest(`${OFFICIAL_ENDPOINTS.leaderboard}?cmd=other`), false);
});

test('summarizes intercepted reports without a signed URL or validator', () => {
  const observation = safeRequestObservation({
    url: () => `${OFFICIAL_ENDPOINTS.leaderboard}?cmd=b&t=l&n=Disposable&p=12345&m=hello`,
    method: () => 'GET',
  });

  assert.deepEqual(observation, {
    endpoint: OFFICIAL_ENDPOINTS.leaderboard,
    method: 'GET',
    operation: 'b',
    trigger: 'l',
    fields: ['cmd', 't', 'n', 'm'],
  });
  assert.doesNotThrow(() => assertCredentialFree(observation));
  assert.throws(
    () => assertCredentialFree('https://example.invalid/?cmd=b&t=l&p=12345'),
    /signed leaderboard URL/,
  );
});

test('builds a deterministic clock injection script with Date and Worker overrides', () => {
  const script = buildClockInitScript();
  assert.match(script, new RegExp(CLOCK_CONTROL));
  assert.match(script, /globalThis\.Date = FakeDate/);
  assert.match(script, /globalThis\.Worker = function Worker/);
  assert.match(script, /runFor: \(totalMs, options = \{\}\)/);
  assert.match(script, new RegExp(String(DEFAULT_CLOCK_START_MS)));
});

test('declares all nine conformance scenarios in the required order', () => {
  assert.deepEqual(
    SCENARIOS.map((scenario) => scenario.id),
    [
      'initial-load',
      'pause',
      'restart',
      'delayed-callback',
      'task-completion',
      'level-up',
      'act-completion',
      'manual-brag',
      'motto-change',
    ],
  );
  assert.deepEqual(
    SCENARIOS.filter((scenario) => scenario.expectedTriggers.length > 0).map(
      (scenario) => scenario.expectedTriggers[0],
    ),
    ['s', 'l', 'a', 'b', 'm'],
  );
});

test('maps bridge events to safe expected request observations', () => {
  const expected = expectedObservationsFromBridgeEvents([
    { trigger: 's' },
    { trigger: 'm' },
  ]);
  assert.deepEqual(expected, [
    {
      endpoint: OFFICIAL_ENDPOINTS.leaderboard,
      method: 'GET',
      operation: 'b',
      trigger: 's',
      fields: [...REPORT_FIELDS],
    },
    {
      endpoint: OFFICIAL_ENDPOINTS.leaderboard,
      method: 'GET',
      operation: 'b',
      trigger: 'm',
      fields: [...REPORT_FIELDS],
    },
  ]);
});

test('builds explicit bridge actions for initial load, bragging, and motto change', () => {
  const byId = Object.fromEntries(SCENARIOS.map((scenario) => [scenario.id, scenario]));
  assert.deepEqual(bridgeActionsForScenario(byId['pause'], 'unused'), []);
  assert.deepEqual(bridgeActionsForScenario(byId['initial-load'], ''), [
    { InitialLoad: { motto: '' } },
  ]);
  assert.deepEqual(bridgeActionsForScenario(byId['manual-brag'], 'Synthetic'), [
    { ManualBrag: { motto: 'Synthetic' } },
  ]);
  assert.deepEqual(bridgeActionsForScenario(byId['motto-change'], MOTTO_CHANGE_TEXT), [
    { MottoChange: { motto: MOTTO_CHANGE_TEXT } },
  ]);
});

test('diffs observed report traces and explains mismatches', () => {
  const expected = expectedObservationsFromBridgeEvents([{ trigger: 'l' }]);
  const passing = compareTraceObservations(expected, expected);
  assert.equal(passing.pass, true);
  assert.deepEqual(passing.differences, []);

  const failing = compareTraceObservations(expected, [
    {
      ...expected[0],
      trigger: 'a',
      fields: ['cmd', 't'],
    },
  ]);
  assert.equal(failing.pass, false);
  assert.match(failing.differences.join('\n'), /trigger/);
  assert.match(failing.differences.join('\n'), /fields/);
});
