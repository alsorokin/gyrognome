#!/usr/bin/env node
/**
 * Explicit, disposable-only browser conformance experiment.
 *
 * This is deliberately not a product transport. Normal Gyrognome builds do
 * not expose its Rust bridge, and report requests are intercepted unless both
 * confirmation flags are present. The bridge receives only a canonical state;
 * the browser's passkey never crosses the ephemeral context boundary.
 */

import { randomUUID } from 'node:crypto';
import { spawn } from 'node:child_process';
import { promises as fs } from 'node:fs';
import { fileURLToPath } from 'node:url';

export const PLAY_URL = 'https://progressquest.com/play/';
export const OFFICIAL_ENDPOINTS = Object.freeze({
  play: PLAY_URL,
  config: 'https://progressquest.com/play/config.js',
  client: 'https://progressquest.com/play/main.js',
  leaderboard: 'https://progressquest.com/alpaquil.php',
});

const forbiddenOption = /^(--(?:character-id|managed-character|managed-id|id))(?:=|$)/;

export function parseOptions(argv) {
  let confirmed = false;
  let submit = false;
  let evidence;

  for (let index = 0; index < argv.length; index += 1) {
    const option = argv[index];
    if (forbiddenOption.test(option)) {
      throw new Error('managed-character input is not accepted');
    }
    if (option === '--confirm-disposable') {
      confirmed = true;
    } else if (option === '--submit') {
      submit = true;
    } else if (option === '--evidence') {
      evidence = argv[++index];
      if (!evidence) throw new Error('--evidence requires a path');
    } else if (option === '--help') {
      return { help: true };
    } else {
      throw new Error(`unknown option: ${option}`);
    }
  }

  if (!confirmed) {
    throw new Error('refusing to run without --confirm-disposable');
  }
  if (submit && !confirmed) {
    throw new Error('--submit requires --confirm-disposable');
  }
  if (evidence?.toLowerCase().endsWith('.pqw')) {
    throw new Error('evidence path must not be a player save');
  }
  return { confirmed, submit, evidence };
}

export function isAllowedOfficialRequest(rawUrl) {
  const url = new URL(rawUrl);
  const endpoint = `${url.origin}${url.pathname}`;
  if (
    url.origin === 'https://progressquest.com' &&
    url.pathname.startsWith('/play/')
  ) {
    return url.search === '';
  }
  if (endpoint === OFFICIAL_ENDPOINTS.leaderboard) {
    return ['create', 'b'].includes(url.searchParams.get('cmd'));
  }
  return false;
}

function isLeaderboardRequest(rawUrl) {
  const url = new URL(rawUrl);
  return (
    `${url.origin}${url.pathname}` === OFFICIAL_ENDPOINTS.leaderboard &&
    ['create', 'b'].includes(url.searchParams.get('cmd'))
  );
}

export function safeRequestObservation(request) {
  const url = new URL(request.url());
  return {
    endpoint: `${url.origin}${url.pathname}`,
    method: request.method(),
    operation: url.searchParams.get('cmd'),
    trigger: url.searchParams.get('t') ?? undefined,
    fields: [...url.searchParams.keys()].filter((key) => key !== 'p'),
  };
}

export function assertCredentialFree(value) {
  const serialized = typeof value === 'string' ? value : JSON.stringify(value);
  const lower = serialized.toLowerCase();
  for (const forbidden of ['passkey', 'original_document', '"document"', '.pqw', '.playwright-mcp']) {
    if (lower.includes(forbidden)) {
      throw new Error(`refusing credential-bearing data (${forbidden})`);
    }
  }
  if (lower.includes('cmd=') && lower.includes('&p=')) {
    throw new Error('refusing a complete signed leaderboard URL');
  }
}

async function callBridge(character, motto) {
  assertCredentialFree(character);
  const input = JSON.stringify({
    character,
    advancement_ms: [],
    motto,
    actions: [{ InitialLoad: { motto } }],
  });
  const stdout = await new Promise((resolve, reject) => {
    const child = spawn(
      'cargo',
      ['run', '--quiet', '--features', 'conformance-bridge', '--', 'conformance-bridge'],
      { stdio: ['pipe', 'pipe', 'ignore'] },
    );
    let output = '';
    child.stdout.setEncoding('utf8');
    child.stdout.on('data', (chunk) => {
      output += chunk;
      if (output.length > 1024 * 1024) child.kill();
    });
    child.on('error', () => reject(new Error('credential-free bridge could not start')));
    child.on('close', (status) => {
      if (status === 0) resolve(output);
      else reject(new Error('credential-free bridge failed'));
    });
    child.stdin.end(input);
  });
  assertCredentialFree(stdout);
  return JSON.parse(stdout);
}

function sanitizedCharacterFromBrowser() {
  const candidate = globalThis.game;
  if (!candidate?.online) throw new Error('browser did not create an online character');
  return JSON.parse(
    JSON.stringify(candidate, (key, value) =>
      ['passkey', 'document', 'original_document'].includes(key.toLowerCase())
        ? undefined
        : value,
    ),
  );
}

async function createDisposableCharacter(page) {
  let stage = 'loading-play-page';
  try {
    await page.goto(PLAY_URL, { waitUntil: 'domcontentloaded' });
    stage = 'opening-roster';
    await page.getByRole('button', { name: 'Play!' }).click();
    stage = 'opening-character-generator';
    await page.getByRole('button', { name: 'Roll One Up' }).click();
    stage = 'selecting-multiplayer';
    await page.getByRole('textbox', { name: 'Name' }).fill(`Conformance-${randomUUID()}`);
    await page.getByRole('radio', { name: 'Multiplayer' }).check();
    stage = 'creating-disposable-online-character';
    await page.getByRole('button', { name: 'Sold!' }).click();
    stage = 'extracting-credential-free-browser-state';
    await page.waitForFunction(() => typeof globalThis.game === 'object' && !!globalThis.game.online);
    return page.evaluate(sanitizedCharacterFromBrowser);
  } catch (error) {
    if (String(error?.message).startsWith('stage:')) {
      throw error;
    }
    throw new Error(`stage:${stage}`);
  }
}

async function runExperiment(options) {
  let stage = 'launching-browser';
  const { chromium } = await import('playwright');
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const page = await context.newPage();
  const observedRequests = [];

  try {
    await context.route('**/*', async (route) => {
      const request = route.request();
      if (request.method() !== 'GET' || !isAllowedOfficialRequest(request.url())) {
        await route.abort('blockedbyclient');
        return;
      }
      if (isLeaderboardRequest(request.url())) {
        if (request.url().includes('cmd=create') && !options.submit) {
          await route.fulfill({
            status: 200,
            contentType: 'text/plain',
            body: 'ok|4242',
          });
          return;
        }
        if (request.url().includes('cmd=b')) {
          observedRequests.push(safeRequestObservation(request));
          if (!options.submit) {
            await route.fulfill({ status: 200, contentType: 'text/plain', body: '' });
            return;
          }
        }
      }
      await route.continue();
    });

    stage = 'creating-disposable-character';
    const character = await createDisposableCharacter(page);
    stage = 'processing-credential-free-state';
    const bridge = await callBridge(character, '');
    stage = 'writing-credential-free-evidence';
    const evidence = {
      format: 'gyrognome-disposable-conformance/v1',
      mode: options.submit ? 'submission-enabled' : 'intercepted',
      source: { client: OFFICIAL_ENDPOINTS.client },
      reports: observedRequests,
      bridge: { eventTriggers: bridge.events.map((event) => event.trigger) },
    };
    assertCredentialFree(evidence);
    if (options.evidence) {
      await fs.writeFile(options.evidence, `${JSON.stringify(evidence, null, 2)}\n`, {
        encoding: 'utf8',
        flag: 'wx',
      });
    }
    process.stdout.write(`${JSON.stringify(evidence)}\n`);
  } catch (error) {
    if (String(error?.message).startsWith('stage:')) {
      throw error;
    }
    throw new Error(`stage:${stage}`);
  } finally {
    await context.close();
    await browser.close();
  }
}

function printHelp() {
  console.log(
    'Usage: node scripts/leaderboard-conformance.mjs --confirm-disposable [--submit] [--evidence path]',
  );
}

async function main() {
  try {
    const options = parseOptions(process.argv.slice(2));
    if (options.help) {
      printHelp();
      return;
    }
    await runExperiment(options);
  } catch (error) {
    // Browser errors may contain request URLs. Do not print their raw text.
    const rawMessage = error instanceof Error ? error.message : '';
    const message =
      rawMessage.includes('--confirm-disposable') ||
      rawMessage.includes('managed-character') ||
      rawMessage === '--evidence requires a path' ||
      rawMessage === 'evidence path must not be a player save'
        ? rawMessage
        : rawMessage.startsWith('stage:')
          ? rawMessage
          : 'safe browser experiment did not complete';
    console.error(`conformance harness refused or failed: ${message}`);
    process.exitCode = 1;
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await main();
}
