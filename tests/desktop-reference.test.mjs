import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import {
  DESKTOP_EVIDENCE_FORMAT,
  DESKTOP_LOAD_SOURCES,
  DESKTOP_PROTOCOL_SOURCE,
  DESKTOP_SOURCE,
  DelphiRandomReference,
  NUMERIC_ASSUMPTIONS,
  applyDesktopLoadDefaults,
  applySpellingPatch,
  callbackRewardVectors,
  completeActReference,
  completionCreditReference,
  dequeueReference,
  desktopFormEncode,
  desktopGuildReference,
  desktopLfsr,
  desktopReportReference,
  freshPrologue,
  levelUpTime,
  levelUpReference,
  loadingAdaptationVectors,
  officialRuntimeCorroborationRecord,
  pascalDiv,
  protocolVectors,
  questMonsterFromPlaceholder,
  randomNumericVectors,
  roundHalfEven,
  spellingNormalizationComparison,
  spellingNormalizationVectors,
  syntheticSmoke,
  timerCallback,
  validateDesktopEvidence,
  weightedStatIndex,
} from '../scripts/desktop-reference.mjs';

function completeEvidence() {
  return {
    format: DESKTOP_EVIDENCE_FORMAT,
    source: DESKTOP_SOURCE,
    reference: {
      implementation: 'scripts/desktop-reference.mjs',
      contentSha256: 'a'.repeat(64),
    },
    production: {
      implementation: 'synthetic-production-test-double',
      contentSha256: 'b'.repeat(64),
    },
    runtimeEquivalence: {
      status: 'unverified',
      officialExecutableCorroboration: 'not-run',
      localContinuationOnly: true,
      classicOnlineEligible: false,
    },
    observations: [
      {
        id: 'synthetic-smoke',
        input: { elapsedMs: 250 },
        output: { creditedMs: 100 },
      },
    ],
  };
}

test('pins the desktop 6.4.4 source and build provenance', () => {
  assert.deepEqual(DESKTOP_SOURCE, {
    repository: 'https://bitbucket.org/grumdrig/pq.git',
    tag: 'v6.4.4',
    commit: '95f5d66f97a3697a7446fba951ab04156c752274',
    capturedOn: '2026-09-25',
    buildToolchain: 'Delphi 6',
    files: {
      'Main.pas': 'fc61edcd1723e4d69724699d1fc60e69f02d0c0530a3c0d90eff446303829f24',
      'Config.pas': 'ebfdf112de08c612d90f1adde5518759d7275b3b3398caf7887ec58caaa4b502',
      'pq.dpr': 'a70454943c349c2c288fec276067827956d5fe2ea14976b3531a955bfbe91db5',
      'pq6-4-4.zip': '0c6982f6e9a4270968dfdec2874dc642957bddbdd55f4517fa013692ffe8e67e',
    },
    officialExecutableSha256:
      'fca7602ed8212bcdafa96d5f35c619142c54396ac54e659c156b967edaeba17c',
    officialExecutableCorroboration: 'not-run',
  });
});

test('runs the source-derived timer smoke deterministically', () => {
  const expected = {
    format: 'gyrognome-desktop-reference-smoke/v1',
    source: DESKTOP_SOURCE,
    scenario: {
      id: 'fill-then-dispatch',
      initialTask: { position: 5_900, max: 6_000 },
      callbacks: [
        {
          elapsedMs: 250,
          position: 6_000,
          creditedMs: 100,
          completionDispatched: false,
        },
        {
          elapsedMs: 75,
          position: 6_000,
          creditedMs: 0,
          completionDispatched: true,
        },
      ],
      finalTask: { position: 6_000, max: 6_000 },
    },
  };

  assert.deepEqual(syntheticSmoke(), expected);
  assert.deepEqual(syntheticSmoke(), expected);
});

test('uses explicit source-derived numeric assumptions', () => {
  assert.deepEqual(NUMERIC_ASSUMPTIONS, {
    randomAlgorithm: 'delphi-6-randseed-lcg-source-derived/v1',
    randomMultiplierHex: '08088405',
    randomIncrement: 1,
    integerDivision: 'truncate-toward-zero',
    round: 'nearest-with-half-to-even',
    exactDelphiCompilerRuntimeEquivalence: 'unverified',
  });
});

test('repeats bounded and 64-bit random continuation from a known seed', () => {
  const first = new DelphiRandomReference(0x12345678n);
  const second = new DelphiRandomReference(0x12345678n);
  const collect = (random) => ({
    bounded: [random.bounded(2n), random.bounded(10n), random.bounded(0xffffffffn)],
    random64: random.random64(),
    random64Below: random.random64Below(1_000_003n),
    state: random.state,
  });
  assert.deepEqual(collect(first), collect(second));
});

test('selects weighted stats repeatably and uses only the first six for weights', () => {
  const first = new DelphiRandomReference(0x0badf00dn);
  const second = new DelphiRandomReference(0x0badf00dn);
  assert.equal(weightedStatIndex([12, 11, 10, 9, 8, 7, 1000], first), 2);
  assert.equal(weightedStatIndex([12, 11, 10, 9, 8, 7, 1], second), 2);
});

test('matches integer division and half-even XP rounding boundaries', () => {
  assert.equal(pascalDiv(5_114n, 1_000n), 5n);
  assert.equal(pascalDiv(-5_114n, 1_000n), -5n);
  assert.equal(roundHalfEven(1_200.5), 1_200);
  assert.equal(roundHalfEven(1_201.5), 1_202);
  assert.deepEqual(levelUpTime(1), { rawSeconds: 1_269, roundedSeconds: 1_269 });
});

test('rejects random and arithmetic values outside declared limits', () => {
  assert.throws(() => new DelphiRandomReference(-1n), RangeError);
  assert.throws(() => new DelphiRandomReference(0x1_0000_0000n), RangeError);
  const random = new DelphiRandomReference(1n);
  assert.throws(() => random.bounded(0n), RangeError);
  assert.throws(() => random.bounded(0x1_0000_0000n), RangeError);
  assert.throws(() => random.random64Below(0n), RangeError);
  assert.throws(() => pascalDiv(1n, 0n), RangeError);
  assert.throws(
    () => weightedStatIndex([4_000_000_000n, 4_000_000_000n, 1, 1, 1, 1], random),
    RangeError,
  );
});

test('replays the committed desktop random and numeric vectors exactly', () => {
  const fixture = JSON.parse(
    readFileSync('tests/fixtures/desktop-random-numeric-vectors.json', 'utf8'),
  );
  assert.deepEqual(randomNumericVectors(), fixture);
});

test('loads evidenced omitted desktop defaults without inventing state', () => {
  assert.deepEqual(applyDesktopLoadDefaults(), {
    spells: [],
    quests: [],
    taskBarPosition: 0,
    questBarPosition: 0,
    plotBarPosition: 0,
    motto: '',
    guild: '',
  });
});

test('preserves both source-derived prologue queues and their final transitions', () => {
  const legacy = freshPrologue('6.2');
  const modern = freshPrologue('6.4.4');
  assert.equal(legacy.queue.at(-1), 'task|2|Loading');
  assert.equal(modern.queue.at(-1), 'plot|2|Loading');

  const legacyFinal = dequeueReference({ ...legacy, queue: [legacy.queue.at(-1)] });
  assert.equal(legacyFinal.actCompleted, false);
  assert.equal(legacyFinal.state.activity, 'Loading...');
  assert.deepEqual(legacyFinal.state.plots, [{ caption: 'Prologue', completed: false }]);

  const modernFinal = dequeueReference({ ...modern, queue: [modern.queue.at(-1)] });
  assert.equal(modernFinal.actCompleted, true);
  assert.equal(modernFinal.state.activity, 'Loading Act I...');
  assert.deepEqual(modernFinal.state.plots, [
    { caption: 'Prologue', completed: true },
    { caption: 'Act I', completed: false },
  ]);
});

test('uses a carried-forward quest placeholder tag rather than its caption', () => {
  assert.equal(
    questMonsterFromPlaceholder({
      questMarker: 'fQuest',
      questTag: 1,
      monsters: ['Synthetic Beast|3|token', 'Synthetic Wyrm|4|scale'],
    }),
    'Synthetic Wyrm|4|scale',
  );
});

test('preserves spell order and the first-row spelling-patch exception', () => {
  assert.deepEqual(DESKTOP_LOAD_SOURCES.spellingPatch, {
    commit: 'ef1ed7f9f5c9ab481a0133f1871871601a9b4a20',
    mainPasGitBlob: '4b18894b851060a2578c45f530f73becc3780000',
    relationship: 'post-v6.4.4-source-history',
  });
  assert.deepEqual(
    applySpellingPatch(['Innoculate', 'Innoculate', 'Tonsilectomy', 'Gyp', 'Shoelaces']),
    ['Innoculate', 'Inoculate', 'Tonsillectomy', 'Gyp', 'Shoelaces'],
  );
});

test('replays the committed desktop loading adaptation vectors exactly', () => {
  const fixture = JSON.parse(
    readFileSync('tests/fixtures/desktop-loading-adaptation-vectors.json', 'utf8'),
  );
  assert.deepEqual(loadingAdaptationVectors(), fixture);
});

test('proves spelling-normalized and canonical inputs have identical protocol state', () => {
  const comparison = spellingNormalizationComparison();
  assert.notDeepEqual(
    comparison.preCorrection.serializedSpells,
    comparison.alreadyCanonical.serializedSpells,
  );
  assert.deepEqual(
    comparison.preCorrection.recordedAdaptations,
    ['load-spelling-patch'],
  );
  assert.deepEqual(comparison.alreadyCanonical.recordedAdaptations, []);
  assert.deepEqual(
    comparison.preCorrection.postLoadCanonicalState,
    comparison.alreadyCanonical.postLoadCanonicalState,
  );
  assert.deepEqual(
    comparison.preCorrection.unsignedRequests,
    comparison.alreadyCanonical.unsignedRequests,
  );
});

test('replays the committed desktop spelling-normalization vectors exactly', () => {
  const fixture = JSON.parse(
    readFileSync('tests/fixtures/desktop-spelling-normalization-vectors.json', 'utf8'),
  );
  assert.deepEqual(spellingNormalizationVectors(), fixture);
  assert.equal(
    fixture.cases.preCorrection.postLoadCanonicalStateSha256,
    fixture.cases.alreadyCanonical.postLoadCanonicalStateSha256,
  );
  assert.deepEqual(
    fixture.cases.preCorrection.unsignedRequestSha256,
    fixture.cases.alreadyCanonical.unsignedRequestSha256,
  );
});

test('credits fractional task maxima with desktop integer division', () => {
  for (const [taskMaxMs, expected] of [[5_114, 5], [5_508, 5], [6_295, 6]]) {
    const kill = completionCreditReference({
      taskMaxMs,
      internalTask: 'kill|Synthetic Beast|1|token',
      experience: 0,
      quest: 0,
      hasQuest: true,
      plot: 0,
      plotMax: 100,
    });
    assert.deepEqual(kill, {
      credit: expected,
      experience: expected,
      quest: expected,
      plot: expected,
    });
    const market = completionCreditReference({
      taskMaxMs,
      internalTask: 'market',
      experience: 0,
      quest: 0,
      hasQuest: true,
      plot: 0,
      plotMax: 100,
    });
    assert.deepEqual(market, {
      credit: expected,
      experience: 0,
      quest: 0,
      plot: expected,
    });
  }
});

test('records level-up event order and random continuation', () => {
  const result = levelUpReference({
    level: 1,
    stats: {
      STR: 12,
      CON: 11,
      DEX: 10,
      INT: 9,
      WIS: 8,
      CHA: 7,
      'HP Max': 20,
      'MP Max': 15,
    },
    spells: [],
    experience: { position: 1_269, max: 1_269 },
  }, new DelphiRandomReference(0x13579bdfn), {
    spells: ['Slime Finger', 'Rabbit Punch', 'Gyp', 'Shoelaces'],
  });
  assert.deepEqual(
    result.events.map((event) => event.type),
    ['level', 'stat', 'stat', 'stat', 'stat', 'spell', 'experience-reset', 'report'],
  );
  assert.equal(result.events.at(-1).trigger, 'l');
  assert.match(result.randomSeed, /^\d+$/);
});

test('gives Act II an item only and later acts item plus equipment', () => {
  const vectors = callbackRewardVectors().observations;
  assert.deepEqual(
    vectors.actTwo.events.map((event) => event.type),
    ['plot-completed', 'plot-added', 'item', 'report'],
  );
  assert.deepEqual(
    vectors.actThree.events.map((event) => event.type),
    ['plot-completed', 'plot-added', 'item', 'equipment', 'report'],
  );
  assert.equal(vectors.actTwo.events[1].caption, 'Act II');
  assert.equal(vectors.actThree.events[1].caption, 'Act III');
});

test('replays the committed desktop callback and reward vectors exactly', () => {
  const fixture = JSON.parse(
    readFileSync('tests/fixtures/desktop-callback-reward-vectors.json', 'utf8'),
  );
  assert.deepEqual(callbackRewardVectors(), fixture);
});

test('constructs revision-8 desktop report fields in source order', () => {
  assert.equal(DESKTOP_PROTOCOL_SOURCE.revision, '8');
  const report = protocolVectors().observations.reports.manual;
  assert.deepEqual(
    report.fieldsBeforeValidator.map(({ name }) => name),
    ['cmd', 't', 'n', 'r', 'c', 'l', 'x', 'i', 'k', 'a', 'h', 'rev'],
  );
  assert.equal(report.fieldsBeforeValidator.some(({ name }) => name === 'z'), false);
  assert.equal(report.fieldsAfterValidator[0].name, 'm');
  assert.equal(report.authentication.credentialsPresent, true);
  assert.equal(report.authentication.preview, 'redacted');
});

test('matches desktop ASCII form escaping and signed validator arithmetic', () => {
  assert.equal(desktopFormEncode("A b&c/!'()*"), "A+b%26c%2F!'()*");
  assert.equal(desktopLfsr('cmd=b&t=l', 4_242), -2_052_713_391);
  assert.throws(() => desktopFormEncode('Non-ASCII ü'), /ASCII/);
  assert.throws(() => desktopLfsr('ü', 1), /ASCII/);
});

test('captures exact level, act, motto, and guild report boundaries', () => {
  const observations = protocolVectors().observations;
  assert.equal(
    observations.reports.level.fieldsBeforeValidator.find(({ name }) => name === 'z').value,
    'Rabbit Punch II',
  );
  assert.equal(
    observations.reports.act.fieldsBeforeValidator.find(({ name }) => name === 'a').value,
    'Act II',
  );
  assert.equal(observations.reports.motto.fieldsAfterValidator[0].value, 'Changed motto!');
  assert.deepEqual(
    observations.guild.fieldsBeforeValidator.map(({ name }) => name),
    ['cmd', 'n', 'r', 'c', 'l', 'h', 'rev', 'guild'],
  );
});

test('builds protocol observations without a destination or complete signed URL', () => {
  const input = {
    trigger: 'b',
    traits: [
      { caption: 'Name', value: 'Synthetic' },
      { caption: 'Race', value: 'Human' },
      { caption: 'Class', value: 'Fighter' },
      { caption: 'Level', value: '1' },
    ],
    experiencePosition: 0,
    equipment: { index: 0, value: 'Stick', caption: 'Weapon' },
    spells: [],
    stats: [
      { caption: 'STR', value: 10 },
      { caption: 'CON', value: 9 },
      { caption: 'DEX', value: 8 },
      { caption: 'INT', value: 7 },
      { caption: 'WIS', value: 6 },
      { caption: 'CHA', value: 5 },
    ],
    plots: ['Prologue'],
    realm: 'Synthetic',
    motto: '',
    authentication: { credentialsPresent: false },
  };
  const report = desktopReportReference(input, 123);
  const guild = desktopGuildReference({
    traits: input.traits,
    realm: input.realm,
    guild: '',
    authentication: input.authentication,
  }, 123);
  for (const observation of [report, guild]) {
    assert.equal(JSON.stringify(observation).includes('://'), false);
    assert.equal(Object.hasOwn(observation, 'signedUrl'), false);
    assert.match(observation.queryBeforeValidator, /^cmd=/);
  }
});

test('replays the committed desktop protocol vectors exactly', () => {
  const fixture = JSON.parse(
    readFileSync('tests/fixtures/desktop-protocol-vectors.json', 'utf8'),
  );
  assert.deepEqual(protocolVectors(), fixture);
});

test('records unavailable official-runtime corroboration without opening online gates', () => {
  const record = officialRuntimeCorroborationRecord();
  assert.equal(record.decision.status, 'not-performed');
  assert.equal(record.decision.suppliedBinaryLaunched, false);
  assert.equal(record.decision.exactDelphiCompilerRuntimeEquivalence, 'unverified');
  assert.equal(record.decision.localContinuationAllowed, true);
  assert.equal(record.decision.classicOnlineDeliveryEnabled, false);
  assert.deepEqual(record.retainedEvidence.runtimeObservations, []);
  assert.deepEqual(record.retainedEvidence.networkObservations, []);
});

test('replays the committed official-runtime limitation record exactly', () => {
  const fixture = JSON.parse(
    readFileSync('tests/fixtures/desktop-official-runtime-corroboration.json', 'utf8'),
  );
  assert.deepEqual(officialRuntimeCorroborationRecord(), fixture);
});

test('rejects invalid synthetic task bars safely', () => {
  assert.throws(() => timerCallback({ position: -1, max: 100 }, 1), RangeError);
  assert.throws(() => timerCallback({ position: 101, max: 100 }, 1), RangeError);
  assert.throws(() => timerCallback({ position: 0, max: 0 }, 1), RangeError);
  assert.throws(() => timerCallback({ position: 0, max: 100 }, 1.5), TypeError);
});

test('validates the separate desktop evidence schema', () => {
  assert.equal(validateDesktopEvidence(completeEvidence()), true);
});

test('rejects incomplete desktop evidence identities and undisclosed limitations', () => {
  for (const mutate of [
    (evidence) => { delete evidence.source.commit; },
    (evidence) => { delete evidence.reference.contentSha256; },
    (evidence) => { delete evidence.production.implementation; },
    (evidence) => { delete evidence.runtimeEquivalence.status; },
    (evidence) => { evidence.runtimeEquivalence.classicOnlineEligible = true; },
    (evidence) => { evidence.observations = []; },
  ]) {
    const evidence = structuredClone(completeEvidence());
    mutate(evidence);
    assert.throws(() => validateDesktopEvidence(evidence));
  }
});

test('rejects credentials, authenticated URLs, raw responses, and raw saves', () => {
  for (const unsafe of [
    { account: 'synthetic-account' },
    { password: 'synthetic-password' },
    { passkey: 12345 },
    { destination: 'https://user:secret@example.invalid/report' },
    { destination: 'https://example.invalid/report?p=12345' },
    { rawResponse: 'unsafe' },
    { rawSave: 'unsafe' },
  ]) {
    const evidence = structuredClone(completeEvidence());
    evidence.observations[0].output = unsafe;
    assert.throws(() => validateDesktopEvidence(evidence), /sensitive or raw/);
  }
});

test('smoke entry point is production-independent and network-free', () => {
  const source = readFileSync('scripts/desktop-reference.mjs', 'utf8');
  for (const forbidden of [
    '../src/',
    'node:child_process',
    'node:http',
    'node:https',
    'node:net',
    'fetch(',
  ]) {
    assert.equal(source.includes(forbidden), false, forbidden);
  }

  const result = spawnSync(
    process.execPath,
    ['scripts/desktop-reference.mjs', '--smoke'],
    { encoding: 'utf8' },
  );
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(JSON.parse(result.stdout), syntheticSmoke());
  assert.equal(result.stderr, '');
});
