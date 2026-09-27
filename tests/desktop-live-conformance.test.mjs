import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { test } from 'node:test';

import {
  ACCOUNT_SCOPE,
  CHARACTER_SCOPE,
  FORMAT,
  dryRun,
  parseOptions,
} from '../scripts/desktop-live-conformance.mjs';

const complete = [
  '--confirm-disposable',
  '--realm',
  'Synthetic Realm',
  '--account-scope',
  ACCOUNT_SCOPE,
  '--character-scope',
  CHARACTER_SCOPE,
  '--operations',
  'manual-brag,motto,guild',
  '--max-active-seconds',
  '300',
  '--confirm-client-handoff',
];

test('requires explicit realm, account, character, operation, bound, and handoff approval', () => {
  for (const omitted of [
    '--confirm-disposable',
    '--realm',
    '--account-scope',
    '--character-scope',
    '--operations',
    '--max-active-seconds',
    '--confirm-client-handoff',
  ]) {
    const index = complete.indexOf(omitted);
    const width = omitted === '--confirm-disposable' || omitted === '--confirm-client-handoff'
      ? 1
      : 2;
    const incomplete = complete.toSpliced(index, width);
    assert.throws(() => parseOptions(incomplete), /refusing experiment without explicit approval/);
  }
});

test('rejects sensitive values and existing identity scopes', () => {
  for (const option of ['--account', '--character', '--passkey', '--password', '--endpoint']) {
    assert.throws(
      () => parseOptions([...complete, option, 'do-not-log']),
      /credentials, endpoints, and existing character identifiers/,
    );
  }
  assert.throws(
    () => parseOptions(complete.toSpliced(complete.indexOf(ACCOUNT_SCOPE), 1, 'existing-account')),
    /new-disposable-account/,
  );
  assert.throws(
    () => parseOptions(complete.toSpliced(complete.indexOf(CHARACTER_SCOPE), 1, 'existing-character')),
    /new-official-client-character/,
  );
});

test('keeps creation and reporting intercepted and records exclusive handoff', () => {
  const options = parseOptions(complete);
  const evidence = dryRun(options);

  assert.equal(evidence.format, FORMAT);
  assert.equal(evidence.mode, 'intercepted-dry-run');
  assert.equal(evidence.transport.creation, 'intercepted');
  assert.equal(evidence.transport.reports, 'intercepted');
  assert.equal(evidence.transport.requestsAttempted, 0);
  assert.equal(evidence.handoff.simultaneousReportersAllowed, false);
  assert.equal(evidence.handoff.nativeReporterStoppedDuringCreation, true);
  assert.equal(evidence.handoff.officialClientStoppedBeforeNativeReporter, true);
  assert.deepEqual(evidence.approval.operations, ['manual-brag', 'motto', 'guild']);

  const serialized = JSON.stringify(evidence);
  assert(!serialized.includes('Synthetic Realm'));
  assert(!serialized.includes('do-not-log'));
});

test('cannot enter live mode without a second confirmation or a reviewed task 9.2 runner', () => {
  assert.throws(
    () => parseOptions([...complete, '--submit']),
    /requires --confirm-live-submission/,
  );
  assert.throws(
    () => parseOptions([...complete, '--submit', '--confirm-live-submission']),
    /live execution is intentionally unavailable/,
  );
});

test('CLI dry run emits sanitized evidence without network activity', () => {
  const result = spawnSync(
    process.execPath,
    ['scripts/desktop-live-conformance.mjs', ...complete],
    { cwd: process.cwd(), encoding: 'utf8' },
  );
  assert.equal(result.status, 0, result.stderr);
  const evidence = JSON.parse(result.stdout);
  assert.equal(evidence.transport.requestsAttempted, 0);
  assert.equal(evidence.handoff.simultaneousReportersAllowed, false);
  assert(!result.stdout.includes('Synthetic Realm'));
});
