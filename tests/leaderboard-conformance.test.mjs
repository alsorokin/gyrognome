import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import {
  CLOCK_CONTROL,
  DEFAULT_CLOCK_START_MS,
  MAX_GUILD_RESPONSE_BYTES,
  GUILD_NORMALIZATION,
  MOTTO_CHANGE_TEXT,
  OFFICIAL_ENDPOINTS,
  REPORT_FIELDS,
  SCENARIOS,
  assertBridgeCredentialFree,
  assertCredentialFree,
  bridgeActionsForScenario,
  buildClassificationQueryUrl,
  buildClockInitScript,
  classifyGuildResponse,
  guildResponseFingerprint,
  exerciseGuildSequence,
  verifyGuildMembership,
  publicGuildId,
  compareTraceObservations,
  expectedObservationsFromBridgeEvents,
  isAllowedClassificationQuery,
  isAllowedOfficialRequest,
  parseLeaderboardClassification,
  parseOptions,
  pollLeaderboardClassification,
  safeRequestObservation,
  summarizeGuildEvidence,
  validateEnrollmentEvidence,
  validateGuildEvidence,
} from '../scripts/leaderboard-conformance.mjs';

function passingGuildEvidence() {
  const evidence = JSON.parse(
    readFileSync('tests/fixtures/enrollment-conformance-evidence.json', 'utf8'),
  );
  const request = {
    endpoint: OFFICIAL_ENDPOINTS.leaderboard,
    method: 'GET',
    operation: 'guild',
    fields: ['cmd', 'n', 'r', 'c', 'l', 'h', 'rev', 'guild'],
  };
  evidence.guild = {
    normalization: GUILD_NORMALIZATION,
    nonEmpty: {
      request,
      outcome: { category: 'accepted', fingerprint: 'a'.repeat(64) },
      browser: { alerted: true, navigated: true, guildEmpty: false },
      serverVerified: true,
      pass: true,
    },
    invalid: {
      request,
      outcome: { category: 'rejected', fingerprint: 'b'.repeat(64) },
      browser: { alerted: true, navigated: false, guildEmpty: false },
      serverVerified: true,
      pass: true,
    },
    empty: {
      request,
      outcome: { category: 'accepted', fingerprint: 'c'.repeat(64) },
      browser: { alerted: true, navigated: true, guildEmpty: true },
      serverVerified: true,
      pass: true,
    },
    order: 'non-empty-before-invalid-before-empty',
    cancellation: { requestAttempts: 0, pass: true },
    cleanup: { browserGuildEmpty: true, pass: true },
    classification: { classification: 'normal', attempts: ['normal'] },
    pass: true,
  };
  return evidence;
}

test('guild fingerprints normalize exact designations without exposing response text', () => {
  const first = guildResponseFingerprint('Old ABC, new AB.', 'ABC', 'AB');
  assert.equal(first, 'c26026a24d960802d64069b942fb3cc8403d4f2974a7e22a541f7804d73c0017');
  assert.equal(guildResponseFingerprint('Joined Guild', '', 'Guild'), 'b199cc402211bdf782855ffcc840e2483ebf2269ce2a9282b5cf3055b6342689');
  assert.match(first, /^[0-9a-f]{64}$/);
  assert.equal(first, guildResponseFingerprint('Old Longer, new Short.', 'Longer', 'Short'));
  assert.equal(
    guildResponseFingerprint('Joined A.* twice A.*', '', 'A.*'),
    guildResponseFingerprint('Joined Z twice Z', '', 'Z'),
  );
  assert.equal(
    guildResponseFingerprint('Joined \u03b1\u03b2', '', '\u03b1\u03b2'),
    guildResponseFingerprint('Joined Guild', '', 'Guild'),
  );
  assert.equal(
    guildResponseFingerprint('Joined Guild', 'Guild', 'Guild'),
    guildResponseFingerprint('Joined Guild', '', 'Guild'),
  );
  assert.notEqual(first, guildResponseFingerprint('Rejected ABC, new AB.', 'ABC', 'AB'));
  assert.doesNotThrow(() => guildResponseFingerprint('x'.repeat(MAX_GUILD_RESPONSE_BYTES)));
  assert.throws(() => guildResponseFingerprint('x'.repeat(MAX_GUILD_RESPONSE_BYTES + 1)));
  assert.throws(() => guildResponseFingerprint('\u03b1'.repeat(MAX_GUILD_RESPONSE_BYTES)));
  assert.throws(() => guildResponseFingerprint(null));
  assert.throws(() => guildResponseFingerprint('', null));
});

test('guild evidence rejects legacy observations and every missing or malformed guild field', () => {
  const valid = passingGuildEvidence();
  assert.equal(validateGuildEvidence(valid), true);
  const bundled = JSON.parse(readFileSync('tests/fixtures/enrollment-conformance-evidence.json', 'utf8'));
  assert.equal(validateGuildEvidence(bundled), true);
  delete bundled.guild.invalid;
  assert.throws(() => validateGuildEvidence(bundled), /incomplete guild/);

  const paths = [];
  const visit = (value, path) => {
    paths.push(path);
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      for (const [key, child] of Object.entries(value)) visit(child, [...path, key]);
    }
  };
  visit(valid.guild, ['guild']);
  for (const path of paths) {
    for (const replacement of [undefined, null, 'malformed']) {
      const candidate = structuredClone(valid);
      const parent = path.slice(0, -1).reduce((value, key) => value[key], candidate);
      if (replacement === undefined) delete parent[path.at(-1)];
      else parent[path.at(-1)] = replacement;
      assert.throws(() => validateGuildEvidence(candidate), path.join('.'));
    }
  }
  for (const mutate of [
    (e) => { e.guild.invalid.outcome.fingerprint = e.guild.nonEmpty.outcome.fingerprint; },
    (e) => { e.guild.invalid.outcome.fingerprint = e.guild.empty.outcome.fingerprint; },
    (e) => { e.guild.nonEmpty.outcome.category = 'message-only'; },
    (e) => { e.guild.invalid.outcome.category = 'accepted'; },
    (e) => { e.guild.empty.outcome.fingerprint = 'A'.repeat(64); },
    (e) => { e.guild.cleanup.pass = false; },
    (e) => { e.guild.classification.attempts = ['cheater', 'normal']; },
    (e) => { e.guild.cancellation.requestAttempts = 1; },
    (e) => { e.guild.invalid.request.fields.push('p'); },
    (e) => { e.guild.invalid.request.endpoint = 'https://example.invalid/'; },
    (e) => { e.guild.invalid.outcome.designation = 'private'; },
    (e) => { e.guild.empty.browser.guildEmpty = false; },
    (e) => { e.scenarios[0].pass = false; },
    (e) => { e.scenarios[0].classification.classification = 'cheater'; },
    (e) => { e.scenarios[0].differences.push('mismatch'); },
    (e) => { e.scenarios.pop(); },
    (e) => { e.enrollment.duplicateName.pass = false; },
    (e) => { e.summary.passed = 0; },
    (e) => { e.guild.nonEmpty.response_body = 'unsafe'; },
  ]) {
    const candidate = structuredClone(valid);
    mutate(candidate);
    assert.throws(() => validateGuildEvidence(candidate));
  }
});

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
    '--guild-designation',
    'TEST',
  ]);
  assert.equal(options.submit, true);
  assert.equal(options.confirmLiveSubmission, true);
});

test('requires a non-empty guild designation for live submission', () => {
  assert.throws(
    () =>
      parseOptions([
        '--confirm-disposable',
        '--submit',
        '--confirm-live-submission',
      ]),
    /--guild-designation/,
  );
  assert.throws(
    () =>
      parseOptions([
        '--confirm-disposable',
        '--submit',
        '--confirm-live-submission',
        '--guild-designation',
        'bad\nvalue',
      ]),
    /control character/,
  );
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
  assert.ok(isAllowedOfficialRequest(`${OFFICIAL_ENDPOINTS.leaderboard}?cmd=guild&guild=TEST`));
  assert.equal(isAllowedOfficialRequest('https://example.invalid/play/'), false);
  assert.equal(isAllowedOfficialRequest('https://progressquest.com/unrelated.js'), false);
  assert.equal(isAllowedOfficialRequest('https://www.progressquest.com/cgi-bin/other.pl'), false);
  assert.equal(isAllowedOfficialRequest(`${OFFICIAL_ENDPOINTS.leaderboard}?cmd=other`), false);
});

test('summarizes guild requests without designation or signed data', () => {
  const observation = safeRequestObservation({
    url: () =>
      `${OFFICIAL_ENDPOINTS.leaderboard}?cmd=guild&n=Disposable&guild=Private&p=12345`,
    method: () => 'GET',
  });

  assert.deepEqual(observation, {
    endpoint: OFFICIAL_ENDPOINTS.leaderboard,
    method: 'GET',
    operation: 'guild',
    trigger: undefined,
    fields: ['cmd', 'n', 'guild'],
  });
  assert.doesNotMatch(JSON.stringify(observation), /Private|12345/);
  assert.doesNotThrow(() => assertCredentialFree(observation));
});

test('classifies guild responses without returning raw content', () => {
  for (const [input, category] of [
    [{ body: 'Joined|/guilds.php' }, 'message-and-navigation'],
    [{ body: 'Rejected' }, 'message-only'],
    [{ body: '|/guilds.php' }, 'navigation-only'],
    [{ body: '' }, 'empty'],
    [{ body: 'a|b|c' }, 'unknown'],
    [{ body: 'ok|https://example.invalid/' }, 'unknown'],
    [{ status: 500, body: 'unsafe response' }, 'http-rejected'],
    [{ error: true }, 'delivery-failed'],
    [{ body: 'x'.repeat(MAX_GUILD_RESPONSE_BYTES + 1) }, 'oversized'],
  ]) {
    const result = classifyGuildResponse(input);
    assert.deepEqual(result, { category });
    assert.equal(Object.hasOwn(result, 'body'), false);
  }
});

test('builds ordered guild evidence and fails closed for cancellation or delivery failures', async (t) => {
  const passing = summarizeGuildEvidence({
    ...passingGuildEvidence().guild,
    cancellationRequestAttempts: 0,
  });

  await t.test('requires complete passing credential-free guild evidence', () => {
    const evidence = passingGuildEvidence();
    assert.equal(validateGuildEvidence(evidence), true);

    for (const mutate of [
      (candidate) => {
        delete candidate.guild.nonEmpty;
      },
      (candidate) => {
        candidate.guild.empty.pass = false;
      },
      (candidate) => {
        candidate.guild.cleanup.browserGuildEmpty = false;
      },
      (candidate) => {
        candidate.guild.classification.classification = 'cheater';
      },
      (candidate) => {
        candidate.guild.nonEmpty.outcome.category = 'unknown';
      },
    ]) {
      const candidate = structuredClone(evidence);
      mutate(candidate);
      assert.throws(() => validateGuildEvidence(candidate), /incomplete guild/);
    }

    const sensitive = structuredClone(evidence);
    sensitive.guild.nonEmpty.response_body = 'unsafe';
    assert.throws(() => validateGuildEvidence(sensitive), /credential-bearing/);
  });
  assert.equal(passing.pass, true);
  assert.equal(passing.order, 'non-empty-before-invalid-before-empty');
  assert.equal(passing.cancellation.pass, true);
  assert.equal(passing.cleanup.pass, true);

  const cancelled = summarizeGuildEvidence({
    ...passing,
    cancellationRequestAttempts: 1,
    nonEmpty: passing.nonEmpty,
    empty: passing.empty,
  });
  assert.equal(cancelled.pass, false);
  const failed = summarizeGuildEvidence({
    ...passing,
    cancellationRequestAttempts: 0,
    nonEmpty: {
      ...passing.nonEmpty,
      outcome: { category: 'delivery-failed' },
    },
    empty: passing.empty,
  });
  assert.equal(failed.pass, false);
  for (const step of ['nonEmpty', 'invalid', 'empty']) {
    const candidate = structuredClone(passing);
    candidate[step].serverVerified = false;
    assert.equal(summarizeGuildEvidence({ ...candidate, cancellationRequestAttempts: 0 }).pass, false);
  }
});

test('guild membership verification uses canonical identity rather than display casing', () => {
  const observation = { found: true, classification: 'normal', guildId: '123' };
  assert.equal(verifyGuildMembership(observation, '123'), true);
  assert.equal(verifyGuildMembership(observation, '456'), false);
  assert.equal(verifyGuildMembership({ ...observation, found: false }, '123'), false);
  assert.equal(verifyGuildMembership({ ...observation, classification: 'cheater' }, '123'), false);
  assert.equal(verifyGuildMembership({ ...observation, guildId: null }, null), true);
  assert.equal(verifyGuildMembership({ ...observation, guildId: null }, '123'), false);
});

test('guild identity accepts official catalog and membership links only', () => {
  assert.equal(publicGuildId('guild.php?id=123'), '123');
  assert.equal(publicGuildId('guilds.php?id=123#123'), '123');
  assert.equal(publicGuildId('https://progressquest.com/guild.php?id=123'), '123');
  for (const href of [
    'https://example.invalid/guild.php?id=123', 'http://progressquest.com/guild.php?id=123',
    'other.php?id=123', 'guild.php?id=invalid', 'guild.php', 'https://[',
  ]) assert.equal(publicGuildId(href), null);
});

test('guild sequence always attempts cleanup once, including failed join or rejection probes', async () => {
  for (const failAt of [null, 'nonEmpty', 'invalid', 'empty']) {
    const calls = [];
    const run = exerciseGuildSequence(async (step) => {
      calls.push(step);
      if (step === failAt) throw new Error('synthetic failure');
      return step;
    });
    if (failAt) await assert.rejects(run, /synthetic failure/);
    else assert.deepEqual(await run, { nonEmpty: 'nonEmpty', invalid: 'invalid', empty: 'empty' });
    assert.deepEqual(calls, failAt === 'nonEmpty' ? ['nonEmpty', 'empty'] : ['nonEmpty', 'invalid', 'empty']);
  }
});

test('summarizes intercepted reports without a signed URL or validator', async (t) => {
  const observation = safeRequestObservation({
    url: () => `${OFFICIAL_ENDPOINTS.leaderboard}?cmd=b&t=l&n=Disposable&p=12345&m=hello`,
    method: () => 'GET',
  });

  await t.test('summarizes creation requests without names or signed data', () => {
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

  await t.test('rejects enrollment evidence containing credentials or raw browser data', () => {
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

  await t.test('allows only the domain online profile through the credential-free bridge', () => {
    assert.doesNotThrow(() =>
      assertBridgeCredentialFree({
        character: {
          profile: { motto: 'A motto', guild: '' },
        },
      }),
    );
    for (const unsafeProfile of [
      'browser-profile',
      { path: '/tmp/browser-profile' },
      { motto: 'A motto', guild: '', userDataDir: '/tmp/browser-profile' },
      { motto: 'passkey', guild: '' },
    ]) {
      assert.throws(() =>
        assertBridgeCredentialFree({ character: { profile: unsafeProfile } }),
      );
    }
    assert.throws(() =>
      assertCredentialFree({ character: { profile: { motto: '', guild: '' } } }),
    );
  });

  await t.test('requires every successful, duplicate, and interrupted enrollment observation', () => {
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

  await t.test('committed enrollment evidence passes the credential-free schema gate', () => {
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
