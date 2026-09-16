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
export const REPORT_FIELDS = Object.freeze([
  'cmd',
  't',
  'n',
  'r',
  'c',
  'l',
  'x',
  'i',
  'z',
  'k',
  'a',
  'h',
  'rev',
  'm',
]);
export const CLOCK_CONTROL = '__gyrognomeConformanceClock';
export const DEFAULT_CLOCK_START_MS = 1_789_462_800_000;
export const DEFAULT_PAUSE_GAP_MS = 30 * 60 * 1000;
export const DEFAULT_DELAYED_CALLBACK_GAP_MS = 10_000;
export const MOTTO_CHANGE_TEXT = 'Deterministic conformance motto';
export const SCENARIOS = Object.freeze([
  Object.freeze({
    id: 'initial-load',
    title: 'initial load',
    fixture: null,
    bridgeAdvancementMs: Object.freeze([]),
    bridgeAction: 'InitialLoad',
    expectedTriggers: Object.freeze(['s']),
  }),
  Object.freeze({
    id: 'pause',
    title: 'pause',
    fixture: 'checkpoint-incomplete-advancement.json',
    browserGapMs: DEFAULT_PAUSE_GAP_MS,
    bridgeAdvancementMs: Object.freeze([]),
    expectedTriggers: Object.freeze([]),
  }),
  Object.freeze({
    id: 'restart',
    title: 'restart',
    fixture: 'checkpoint-incomplete-advancement.json',
    browserGapMs: DEFAULT_PAUSE_GAP_MS,
    bridgeAdvancementMs: Object.freeze([]),
    expectedTriggers: Object.freeze([]),
  }),
  Object.freeze({
    id: 'delayed-callback',
    title: 'delayed callback',
    fixture: 'checkpoint-incomplete-advancement.json',
    browserGapMs: DEFAULT_DELAYED_CALLBACK_GAP_MS,
    bridgeAdvancementMs: Object.freeze([100]),
    expectedTriggers: Object.freeze([]),
  }),
  Object.freeze({
    id: 'task-completion',
    title: 'task completion',
    fixture: 'checkpoint-completed-task.json',
    bridgeAdvancementMs: Object.freeze([9451]),
    expectedTriggers: Object.freeze([]),
    flushCompletion: true,
  }),
  Object.freeze({
    id: 'level-up',
    title: 'level-up',
    fixture: 'checkpoint-level-up.json',
    bridgeAdvancementMs: Object.freeze([1000]),
    expectedTriggers: Object.freeze(['l']),
    flushCompletion: true,
  }),
  Object.freeze({
    id: 'act-completion',
    title: 'act completion',
    fixture: 'checkpoint-act.json',
    bridgeAdvancementMs: Object.freeze([1000]),
    expectedTriggers: Object.freeze(['a']),
    flushCompletion: true,
  }),
  Object.freeze({
    id: 'manual-brag',
    title: 'manual bragging',
    fixture: 'checkpoint-completed-task.json',
    bridgeAdvancementMs: Object.freeze([]),
    bridgeAction: 'ManualBrag',
    expectedTriggers: Object.freeze(['b']),
  }),
  Object.freeze({
    id: 'motto-change',
    title: 'motto change',
    fixture: 'checkpoint-completed-task.json',
    bridgeAdvancementMs: Object.freeze([]),
    bridgeAction: 'MottoChange',
    actionMotto: MOTTO_CHANGE_TEXT,
    expectedTriggers: Object.freeze(['m']),
  }),
]);

const forbiddenOption = /^(--(?:character-id|managed-character|managed-id|id))(?:=|$)/;
const FIXTURE_DIRECTORY = new URL('../tests/fixtures/', import.meta.url);

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

export function buildClockInitScript({
  controlName = CLOCK_CONTROL,
  startTimeMs = DEFAULT_CLOCK_START_MS,
} = {}) {
  return `(() => {
    const controlName = ${JSON.stringify(controlName)};
    const initialNow = ${JSON.stringify(startTimeMs)};
    const RealDate = Date;
    const RealWorker = globalThis.Worker;
    let now = initialNow;

    function FakeDate(...args) {
      if (new.target) {
        return args.length === 0
          ? Reflect.construct(RealDate, [now], new.target)
          : Reflect.construct(RealDate, args, new.target);
      }
      return args.length === 0 ? RealDate(now).toString() : RealDate(...args);
    }

    Object.setPrototypeOf(FakeDate, RealDate);
    FakeDate.prototype = RealDate.prototype;
    FakeDate.now = () => now;
    FakeDate.parse = RealDate.parse;
    FakeDate.UTC = RealDate.UTC;

    class FakeClockWorker {
      constructor(url, options) {
        this.url = String(url);
        this.options = options;
        this.running = false;
        this.lasttick = now;
        this.listeners = [];
        this.onerror = null;
        this.onmessage = null;
      }

      addEventListener(type, listener) {
        if (type === 'message') this.listeners.push(listener);
      }

      removeEventListener(type, listener) {
        if (type !== 'message') return;
        this.listeners = this.listeners.filter((candidate) => candidate !== listener);
      }

      dispatchEvent(event) {
        if (event?.type !== 'message') return true;
        this._emit(event.data);
        return true;
      }

      postMessage(message) {
        if (message === 'start') {
          this.lasttick = now;
          this.running = true;
        } else if (message === 'stop') {
          this.running = false;
        }
      }

      terminate() {
        this.running = false;
      }

      _emit(data = 'tick') {
        if (!this.running) return;
        const event = { data, type: 'message', target: this };
        for (const listener of this.listeners) listener.call(this, event);
        if (typeof this.onmessage === 'function') this.onmessage.call(this, event);
      }
    }

    globalThis.Date = FakeDate;
    globalThis.Worker = function Worker(url, options) {
      if (String(url).endsWith('clock.js')) {
        return new FakeClockWorker(url, options);
      }
      return new RealWorker(url, options);
    };
    globalThis.Worker.prototype = RealWorker?.prototype;

    globalThis[controlName] = {
      now: () => now,
      set: (value) => {
        now = Number(value);
        return now;
      },
      advance: (delta) => {
        now += Number(delta);
        return now;
      },
      dispatchTick: (delta = 0) => {
        now += Number(delta);
        if (globalThis.clock && typeof globalThis.clock._emit === 'function') {
          globalThis.clock._emit('tick');
        }
        return now;
      },
      runFor: (totalMs, options = {}) => {
        let remaining = Number(totalMs);
        const maxStepMs = Number(options.maxStepMs ?? 100);
        while (remaining > 0) {
          const step = Math.min(maxStepMs, remaining);
          now += step;
          if (globalThis.clock && typeof globalThis.clock._emit === 'function') {
            globalThis.clock._emit('tick');
          }
          remaining -= step;
        }
        if (options.flushCompletion && globalThis.clock && typeof globalThis.clock._emit === 'function') {
          globalThis.clock._emit('tick');
        }
        return now;
      },
    };
  })();`;
}

export function bridgeActionsForScenario(scenario, motto = '') {
  if (!scenario.bridgeAction) return [];
  return [{ [scenario.bridgeAction]: { motto } }];
}

export function expectedObservationsFromBridgeEvents(events) {
  return events.map((event) => ({
    endpoint: OFFICIAL_ENDPOINTS.leaderboard,
    method: 'GET',
    operation: 'b',
    trigger: event.trigger,
    fields: [...REPORT_FIELDS],
  }));
}

export function compareTraceObservations(expected, observed) {
  const differences = [];
  if (expected.length !== observed.length) {
    differences.push(`expected ${expected.length} report(s), observed ${observed.length}`);
  }
  const count = Math.max(expected.length, observed.length);
  for (let index = 0; index < count; index += 1) {
    const expectedEntry = expected[index];
    const observedEntry = observed[index];
    if (!expectedEntry) {
      differences.push(`unexpected report ${index + 1}: ${JSON.stringify(observedEntry)}`);
      continue;
    }
    if (!observedEntry) {
      differences.push(`missing report ${index + 1}: ${JSON.stringify(expectedEntry)}`);
      continue;
    }
    for (const field of ['endpoint', 'method', 'operation', 'trigger']) {
      if (expectedEntry[field] !== observedEntry[field]) {
        differences.push(
          `report ${index + 1} ${field}: expected ${JSON.stringify(expectedEntry[field])}, observed ${JSON.stringify(observedEntry[field])}`,
        );
      }
    }
    if (JSON.stringify(expectedEntry.fields) !== JSON.stringify(observedEntry.fields)) {
      differences.push(
        `report ${index + 1} fields: expected ${JSON.stringify(expectedEntry.fields)}, observed ${JSON.stringify(observedEntry.fields)}`,
      );
    }
  }
  return { pass: differences.length === 0, differences };
}

export function summarizeCharacter(character) {
  return {
    name: character?.Traits?.Name ?? character?.traits?.Name,
    level: character?.Traits?.Level ?? character?.traits?.Level,
    elapsed: character?.elapsed ?? character?.activity?.elapsed,
    task: character?.task ?? character?.activity?.task,
    bestPlot: character?.bestplot ?? character?.plot?.bestplot,
  };
}

function bridgeInputForScenario(character, scenario, motto) {
  const actionMotto = scenario.actionMotto ?? motto;
  const actions = bridgeActionsForScenario(scenario, actionMotto);
  return {
    character,
    advancement_ms: [...scenario.bridgeAdvancementMs],
    motto: actionMotto,
    actions,
  };
}

async function callBridge(input) {
  assertCredentialFree(input);
  const stdin = JSON.stringify(input);
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
    child.stdin.end(stdin);
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

async function waitForGame(page) {
  await page.waitForFunction(() => typeof globalThis.game === 'object' && !!globalThis.game.online);
}

async function loadFixtureState(page, state) {
  assertCredentialFree(state);
  await page.evaluate((nextState) => {
    if (!globalThis.game?.online?.passkey) {
      throw new Error('missing disposable online passkey inside browser context');
    }
    const online = structuredClone(globalThis.game.online);
    const currentName = globalThis.game.Traits?.Name;
    const loaded = structuredClone(nextState);
    loaded.online = online;
    loaded.Traits.Name = currentName;
    loaded.saveName = `${currentName} [${online.realm}]`;
    loaded.motto = loaded.motto ?? '';
    LoadGame(loaded);
    StopTimer();
  }, state);
}

async function persistCurrentGame(page) {
  await page.evaluate(
    () =>
      new Promise((resolve, reject) => {
        try {
          SaveGame(resolve);
        } catch (error) {
          reject(String(error));
        }
      }),
  );
}

async function stopTimer(page) {
  await page.evaluate(() => {
    if (typeof StopTimer === 'function') StopTimer();
  });
}

async function startTimer(page) {
  await page.evaluate(() => {
    if (typeof StartTimer === 'function') StartTimer();
  });
}

async function currentMotto(page) {
  return page.evaluate(() => globalThis.game?.motto || '');
}

async function currentCharacter(page) {
  return page.evaluate(sanitizedCharacterFromBrowser);
}

async function runClock(page, method, ...args) {
  return page.evaluate(
    ({ controlName, methodName, argumentsList }) => {
      const controls = globalThis[controlName];
      if (!controls || typeof controls[methodName] !== 'function') {
        throw new Error(`missing clock control method: ${methodName}`);
      }
      return controls[methodName](...argumentsList);
    },
    { controlName: CLOCK_CONTROL, methodName: method, argumentsList: args },
  );
}

async function waitForObservedCount(observedRequests, expectedCount) {
  const deadline = Date.now() + 5_000;
  while (observedRequests.length < expectedCount) {
    if (Date.now() >= deadline) {
      throw new Error('timed out waiting for intercepted report');
    }
    await new Promise((resolve) => setTimeout(resolve, 25));
  }
}

async function settleNoReport() {
  await new Promise((resolve) => setTimeout(resolve, 100));
}

async function loadScenarioFixture(fixtureName) {
  const path = new URL(fixtureName, FIXTURE_DIRECTORY);
  const content = await fs.readFile(path, 'utf8');
  const fixture = JSON.parse(content);
  return fixture.initial;
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
    await waitForGame(page);
    await stopTimer(page);
    return currentCharacter(page);
  } catch (error) {
    if (String(error?.message).startsWith('stage:')) {
      throw error;
    }
    throw new Error(`stage:${stage}`);
  }
}

async function executeScenario(page, scenario) {
  switch (scenario.id) {
    case 'pause':
      await runClock(page, 'advance', scenario.browserGapMs);
      return;
    case 'restart':
      await persistCurrentGame(page);
      await runClock(page, 'advance', scenario.browserGapMs);
      await page.reload({ waitUntil: 'domcontentloaded' });
      await waitForGame(page);
      await stopTimer(page);
      return;
    case 'delayed-callback':
      // Resume the real worker so a genuine Timer1Timer callback fires and
      // applies its own >100ms cap, rather than exercising a stopped clock
      // (which the "pause"/"restart" scenarios already cover).
      await startTimer(page);
      await runClock(page, 'dispatchTick', scenario.browserGapMs);
      return;
    case 'task-completion':
    case 'level-up':
    case 'act-completion':
      // loadFixtureState() stops the real worker; resume it here so the
      // injected clock's dispatched ticks actually reach Timer1Timer().
      await startTimer(page);
      await runClock(page, 'runFor', scenario.bridgeAdvancementMs[0], {
        maxStepMs: 100,
        flushCompletion: !!scenario.flushCompletion,
      });
      return;
    case 'manual-brag':
      await page.evaluate(() => Brag('b'));
      return;
    case 'motto-change':
      await page.evaluate((motto) => {
        globalThis.game.motto = motto;
        Brag('m');
      }, scenario.actionMotto);
      return;
    default:
      throw new Error(`unknown scenario: ${scenario.id}`);
  }
}

async function runInitialLoadScenario(page, observedRequests) {
  const requestStart = observedRequests.length;
  const character = await createDisposableCharacter(page);
  const motto = await currentMotto(page);
  const scenario = SCENARIOS[0];
  const bridge = await callBridge(bridgeInputForScenario(character, scenario, motto));
  const expected = expectedObservationsFromBridgeEvents(bridge.events);
  await waitForObservedCount(observedRequests, requestStart + expected.length);
  const observed = observedRequests.slice(requestStart);
  const comparison = compareTraceObservations(expected, observed);
  const result = {
    id: scenario.id,
    title: scenario.title,
    pass: comparison.pass,
    browserGapMs: 0,
    bridgeAdvancementMs: [],
    snapshot: summarizeCharacter(character),
    expected,
    observed,
    bridge: {
      eventTriggers: bridge.events.map((event) => event.trigger),
    },
    differences: comparison.differences,
  };
  assertCredentialFree(result);
  return result;
}

async function runFixtureScenario(page, scenario, observedRequests) {
  const fixtureState = await loadScenarioFixture(scenario.fixture);
  await loadFixtureState(page, fixtureState);
  const snapshot = await currentCharacter(page);
  const motto = await currentMotto(page);
  const bridge = await callBridge(bridgeInputForScenario(snapshot, scenario, motto));
  const expected = expectedObservationsFromBridgeEvents(bridge.events);
  const requestStart = observedRequests.length;
  await executeScenario(page, scenario);
  if (expected.length > 0) {
    await waitForObservedCount(observedRequests, requestStart + expected.length);
  } else {
    await settleNoReport();
  }
  const observed = observedRequests.slice(requestStart);
  const comparison = compareTraceObservations(expected, observed);
  const result = {
    id: scenario.id,
    title: scenario.title,
    pass: comparison.pass,
    browserGapMs: scenario.browserGapMs ?? 0,
    bridgeAdvancementMs: [...scenario.bridgeAdvancementMs],
    snapshot: summarizeCharacter(snapshot),
    expected,
    observed,
    bridge: {
      eventTriggers: bridge.events.map((event) => event.trigger),
    },
    differences: comparison.differences,
  };
  assertCredentialFree(result);
  return result;
}

async function runExperiment(options) {
  let stage = 'launching-browser';
  const { chromium } = await import('playwright');
  const browser = await chromium.launch();
  const context = await browser.newContext();
  await context.addInitScript(buildClockInitScript());
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

    stage = 'running-paired-scenarios';
    const scenarios = [];
    scenarios.push(await runInitialLoadScenario(page, observedRequests));
    for (const scenario of SCENARIOS.slice(1)) {
      scenarios.push(await runFixtureScenario(page, scenario, observedRequests));
    }

    stage = 'writing-credential-free-evidence';
    const evidence = {
      format: 'gyrognome-disposable-conformance/v1',
      mode: options.submit ? 'submission-enabled' : 'intercepted',
      source: { client: OFFICIAL_ENDPOINTS.client },
      scenarios,
      summary: {
        total: scenarios.length,
        passed: scenarios.filter((scenario) => scenario.pass).length,
        failed: scenarios.filter((scenario) => !scenario.pass).map((scenario) => scenario.id),
      },
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
