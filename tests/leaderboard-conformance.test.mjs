import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
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
  buildClassificationQueryUrl,
  buildClockInitScript,
  compareTraceObservations,
  expectedObservationsFromBridgeEvents,
  isAllowedClassificationQuery,
  isAllowedOfficialRequest,
  parseLeaderboardClassification,
  parseOptions,
  pollLeaderboardClassification,
  safeRequestObservation,
  validateEnrollmentEvidence,
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

test('requires a second explicit confirmation before submitting live reports', () => {
  assert.throws(
    () => parseOptions(['--confirm-disposable', '--submit']),
    /--confirm-live-submission/,
  );
  const options = parseOptions([
    '--confirm-disposable',
    '--submit',
    '--confirm-live-submission',
  ]);
  assert.equal(options.submit, true);
  assert.equal(options.confirmLiveSubmission, true);
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

  test('summarizes creation requests without names or signed data', () => {
    const observation = safeRequestObservation({
      url: () =>
        `${OFFICIAL_ENDPOINTS.leaderboard}?cmd=create&name=Disposable&realm=1&rev=6`,
      method: () => 'GET',
    });

    assert.deepEqual(observation, {
      endpoint: OFFICIAL_ENDPOINTS.leaderboard,
      method: 'GET',
      operation: 'create',
      trigger: undefined,
      fields: ['cmd', 'name', 'realm', 'rev'],
    });
    assert.doesNotThrow(() => assertCredentialFree(observation));
  });

  test('rejects enrollment evidence containing credentials or raw browser data', () => {
    for (const unsafeEvidence of [
      { passkey: 'unsafe' },
      { response: 'unsafe' },
      { response_body: 'unsafe' },
      { profile: 'unsafe' },
      { raw_save: 'unsafe' },
      'https://example.invalid/?cmd=create&p=12345',
    ]) {
      assert.throws(() => assertCredentialFree(unsafeEvidence));
    }
  });

  test('requires every successful, duplicate, and interrupted enrollment observation', () => {
    const evidence = {
      format: 'gyrognome-disposable-conformance/v2',
      mode: 'submission-enabled',
      source: {
        client: OFFICIAL_ENDPOINTS.client,
        revision: '6',
        content_sha256: 'safe-source-hash',
      },
      enrollment: {
        successfulCreation: {
          outcome: 'success',
          creation: { operation: 'create' },
          initialReport: { operation: 'b', trigger: 's' },
          order: 'create-before-initial-report',
          pass: true,
        },
        duplicateName: {
          outcome: 'rejected',
          creation: { operation: 'create' },
          creationAttempts: 1,
          initialReportAttempts: 0,
          additionalOnlineIdentity: false,
          pass: true,
        },
        interruptedEnrollment: {
          outcome: 'unconfirmed',
          creations: [{ operation: 'create' }],
          initialReportAttempts: 0,
          pass: true,
        },
      },
      summary: { total: 3, passed: 3, failed: [] },
    };
    assert.equal(validateEnrollmentEvidence(evidence), true);
    delete evidence.enrollment.duplicateName;
    assert.throws(() => validateEnrollmentEvidence(evidence), /incomplete enrollment/);
  });

  test('committed enrollment evidence passes the credential-free schema gate', () => {
    const evidence = JSON.parse(
      readFileSync('tests/fixtures/enrollment-conformance-evidence.json', 'utf8'),
    );
    assert.equal(validateEnrollmentEvidence(evidence), true);
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

test('classification query is scoped to the public realm page and only a name filter', () => {
  const url = buildClassificationQueryUrl('Conformance-example');
  assert.equal(url, `${OFFICIAL_ENDPOINTS.leaderboard}?name=Conformance-example`);
  assert.ok(isAllowedClassificationQuery(url));
  assert.equal(
    isAllowedClassificationQuery(`${OFFICIAL_ENDPOINTS.leaderboard}?name=x&cheaters=1`),
    false,
  );
  assert.equal(isAllowedClassificationQuery('https://example.invalid/?name=x'), false);
});

test('parses Hall of Fame and Hall of Infamy classification pages', () => {
  assert.deepEqual(
    parseLeaderboardClassification('<h1>Hall of Fame</h1><tr class=bob><td>x</table>'),
    { classification: 'normal' },
  );
  assert.deepEqual(
    parseLeaderboardClassification('<h1>Hall of Infamy</h1><tr class=bob><td>x</table>'),
    { classification: 'cheater' },
  );
  assert.deepEqual(parseLeaderboardClassification('<h1>Hall of Fame</h1><table></table>'), {
    classification: 'not-found',
  });
});

test('polls the leaderboard on documented bounds and resolves once classified', async () => {
  let calls = 0;
  const fetchImpl = async () => {
    calls += 1;
    const html =
      calls < 3
        ? '<h1>Hall of Fame</h1><table></table>'
        : '<h1>Hall of Fame</h1><tr class=bob><td>x</table>';
    return { ok: true, text: async () => html };
  };
  const sleeps = [];
  const result = await pollLeaderboardClassification('Conformance-example', {
    fetchImpl,
    sleep: async (ms) => sleeps.push(ms),
  });
  assert.deepEqual(result, { classification: 'normal', attempts: ['not-found', 'not-found', 'normal'] });
  assert.equal(calls, 3);
  assert.deepEqual(sleeps, [5000, 5000]);
});

test('treats a classification that never resolves within bounds as inconclusive, not a pass', async () => {
  const fetchImpl = async () => ({ ok: true, text: async () => '<h1>Hall of Fame</h1><table></table>' });
  let now = 0;
  const originalNow = Date.now;
  Date.now = () => now;
  try {
    const result = await pollLeaderboardClassification('Conformance-example', {
      fetchImpl,
      timeoutMs: 10_000,
      intervalMs: 5_000,
      sleep: async () => {
        now += 5_000;
      },
    });
    assert.equal(result.classification, 'inconclusive');
  } finally {
    Date.now = originalNow;
  }
});
