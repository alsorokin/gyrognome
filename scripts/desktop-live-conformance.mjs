#!/usr/bin/env node
/**
 * Credential-free planning guard for a separately approved classic desktop
 * experiment. This file intentionally contains no client automation or network
 * transport. Its only executable mode is an intercepted dry run.
 */

import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';

export const FORMAT = 'gyrognome-desktop-live-experiment-dry-run/v1';
export const ACCOUNT_SCOPE = 'new-disposable-account';
export const CHARACTER_SCOPE = 'new-official-client-character';
export const OPERATIONS = Object.freeze([
  'automatic-level',
  'automatic-act',
  'manual-brag',
  'motto',
  'guild',
]);
export const MAX_ACTIVE_SECONDS = 3_600;

const sensitiveOption = /^(--(?:account|character|credential|endpoint|passkey|password))(?:=|$)/;

function optionValue(argv, index, option) {
  const value = argv[index + 1];
  if (!value || value.startsWith('--')) {
    throw new Error(`${option} requires a value`);
  }
  return value;
}

function validateText(value, option) {
  if (value.length > 128 || [...value].some((character) => /\p{Cc}/u.test(character))) {
    throw new Error(`${option} is invalid`);
  }
}

export function parseOptions(argv) {
  const options = {
    confirmDisposable: false,
    confirmClientHandoff: false,
    confirmLiveSubmission: false,
    submit: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const option = argv[index];
    if (sensitiveOption.test(option)) {
      throw new Error('credentials, endpoints, and existing character identifiers are not accepted');
    }
    switch (option) {
      case '--confirm-disposable':
        options.confirmDisposable = true;
        break;
      case '--confirm-client-handoff':
        options.confirmClientHandoff = true;
        break;
      case '--confirm-live-submission':
        options.confirmLiveSubmission = true;
        break;
      case '--submit':
        options.submit = true;
        break;
      case '--realm':
        options.realm = optionValue(argv, index, option);
        index += 1;
        break;
      case '--account-scope':
        options.accountScope = optionValue(argv, index, option);
        index += 1;
        break;
      case '--character-scope':
        options.characterScope = optionValue(argv, index, option);
        index += 1;
        break;
      case '--operations':
        options.operations = optionValue(argv, index, option)
          .split(',')
          .filter(Boolean);
        index += 1;
        break;
      case '--max-active-seconds':
        options.maxActiveSeconds = Number(optionValue(argv, index, option));
        index += 1;
        break;
      case '--help':
        return { help: true };
      default:
        throw new Error(`unknown option: ${option}`);
    }
  }

  const missing = [];
  if (!options.confirmDisposable) missing.push('--confirm-disposable');
  if (!options.realm) missing.push('--realm');
  if (!options.accountScope) missing.push('--account-scope');
  if (!options.characterScope) missing.push('--character-scope');
  if (!options.operations?.length) missing.push('--operations');
  if (!options.maxActiveSeconds) missing.push('--max-active-seconds');
  if (!options.confirmClientHandoff) missing.push('--confirm-client-handoff');
  if (missing.length) {
    throw new Error(`refusing experiment without explicit approval: ${missing.join(', ')}`);
  }

  validateText(options.realm, '--realm');
  if (options.accountScope !== ACCOUNT_SCOPE) {
    throw new Error(`--account-scope must be ${ACCOUNT_SCOPE}`);
  }
  if (options.characterScope !== CHARACTER_SCOPE) {
    throw new Error(`--character-scope must be ${CHARACTER_SCOPE}`);
  }
  if (
    !Number.isSafeInteger(options.maxActiveSeconds)
    || options.maxActiveSeconds < 1
    || options.maxActiveSeconds > MAX_ACTIVE_SECONDS
  ) {
    throw new Error(`--max-active-seconds must be between 1 and ${MAX_ACTIVE_SECONDS}`);
  }
  const invalidOperation = options.operations.find(
    (operation) => !OPERATIONS.includes(operation),
  );
  if (invalidOperation) {
    throw new Error('one or more --operations values are unsupported');
  }
  options.operations = [...new Set(options.operations)];

  if (options.submit && !options.confirmLiveSubmission) {
    throw new Error('--submit requires --confirm-live-submission');
  }
  if (options.submit) {
    throw new Error(
      'live execution is intentionally unavailable in the dry-run harness; obtain separate approval and use the reviewed task 9.2 procedure',
    );
  }
  return options;
}

function fingerprint(value) {
  return createHash('sha256').update(value, 'utf8').digest('hex');
}

export function dryRun(options) {
  if (options.help) return { help: true };
  return {
    format: FORMAT,
    mode: 'intercepted-dry-run',
    approval: {
      disposableIdentity: true,
      realmSha256: fingerprint(options.realm),
      accountScope: ACCOUNT_SCOPE,
      characterScope: CHARACTER_SCOPE,
      operations: options.operations,
      maxActiveSeconds: options.maxActiveSeconds,
    },
    handoff: {
      officialClientCreatesIdentity: true,
      nativeReporterStoppedDuringCreation: true,
      officialClientStoppedBeforeNativeReporter: true,
      simultaneousReportersAllowed: false,
    },
    transport: {
      creation: 'intercepted',
      reports: 'intercepted',
      requestsAttempted: 0,
    },
    cleanupRequired: true,
  };
}

export function usage() {
  return [
    'usage: node scripts/desktop-live-conformance.mjs',
    '  --confirm-disposable',
    '  --realm <approved-realm>',
    `  --account-scope ${ACCOUNT_SCOPE}`,
    `  --character-scope ${CHARACTER_SCOPE}`,
    `  --operations <${OPERATIONS.join('|')},...>`,
    `  --max-active-seconds <1-${MAX_ACTIVE_SECONDS}>`,
    '  --confirm-client-handoff',
    '',
    'The default and only implemented mode is transport-intercepted dry run.',
    'Live execution requires separate task 9.2 approval and is not implemented here.',
  ].join('\n');
}

async function main(argv) {
  const options = parseOptions(argv);
  if (options.help) {
    console.log(usage());
    return;
  }
  console.log(JSON.stringify(dryRun(options), null, 2));
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  main(process.argv.slice(2)).catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
