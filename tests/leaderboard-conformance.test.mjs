import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import test from 'node:test';

import {
  OFFICIAL_ENDPOINTS,
  assertCredentialFree,
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

  test('treats creation as an allowed disposable-browser operation', () => {
    assert.ok(isAllowedOfficialRequest(`${OFFICIAL_ENDPOINTS.leaderboard}?cmd=create`));
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
