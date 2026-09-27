#!/usr/bin/env node

import { createHash } from 'node:crypto';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

export const DESKTOP_SOURCE = Object.freeze({
  repository: 'https://bitbucket.org/grumdrig/pq.git',
  tag: 'v6.4.4',
  commit: '95f5d66f97a3697a7446fba951ab04156c752274',
  capturedOn: '2026-09-25',
  buildToolchain: 'Delphi 6',
  files: Object.freeze({
    'Main.pas': 'fc61edcd1723e4d69724699d1fc60e69f02d0c0530a3c0d90eff446303829f24',
    'Config.pas': 'ebfdf112de08c612d90f1adde5518759d7275b3b3398caf7887ec58caaa4b502',
    'pq.dpr': 'a70454943c349c2c288fec276067827956d5fe2ea14976b3531a955bfbe91db5',
    'pq6-4-4.zip': '0c6982f6e9a4270968dfdec2874dc642957bddbdd55f4517fa013692ffe8e67e',
  }),
  officialExecutableSha256:
    'fca7602ed8212bcdafa96d5f35c619142c54396ac54e659c156b967edaeba17c',
  officialExecutableCorroboration: 'not-run',
});

export const MAX_CALLBACK_ELAPSED_MS = 100;
export const DESKTOP_EVIDENCE_FORMAT = 'gyrognome-desktop-reference-evidence/v1';
export const NUMERIC_ASSUMPTIONS = Object.freeze({
  randomAlgorithm: 'delphi-6-randseed-lcg-source-derived/v1',
  randomMultiplierHex: '08088405',
  randomIncrement: 1,
  integerDivision: 'truncate-toward-zero',
  round: 'nearest-with-half-to-even',
  exactDelphiCompilerRuntimeEquivalence: 'unverified',
});
export const DESKTOP_LOAD_SOURCES = Object.freeze({
  legacy62: Object.freeze({
    tag: 'v6.2',
    commit: '5e7e820533146f179174c97b592cee87db1f6f10',
    mainPasGitBlob: '4f4ee563577407de34d29fa5058da73b9c2271b9',
  }),
  spellingPatch: Object.freeze({
    commit: 'ef1ed7f9f5c9ab481a0133f1871871601a9b4a20',
    mainPasGitBlob: '4b18894b851060a2578c45f530f73becc3780000',
    relationship: 'post-v6.4.4-source-history',
  }),
});
export const DESKTOP_PROTOCOL_SOURCE = Object.freeze({
  revision: '8',
  mainPasSha256: DESKTOP_SOURCE.files['Main.pas'],
  newGuyPasSha256: 'fd3443ea80c5e7827d10b62649ec1e8ac23c5d75ea8d89acd2b763581de9d243',
  encodingComponent: 'Delphi 6 TNMURL.Encode',
  supportedEncoding: 'ASCII application/x-www-form-urlencoded',
});

const UINT32_MODULUS = 1n << 32n;
const UINT32_MAX = UINT32_MODULUS - 1n;
const INT64_MAX = (1n << 63n) - 1n;
const RANDOM_MULTIPLIER = 0x08088405n;

function requireInteger(value, name) {
  if (!Number.isSafeInteger(value)) {
    throw new TypeError(`${name} must be a safe integer`);
  }
}

function requireBigIntRange(value, minimum, maximum, name) {
  if (typeof value !== 'bigint' || value < minimum || value > maximum) {
    throw new RangeError(`${name} is outside the supported range`);
  }
}

export class DelphiRandomReference {
  #state;

  constructor(seed) {
    const value = typeof seed === 'bigint' ? seed : BigInt(seed);
    requireBigIntRange(value, 0n, UINT32_MAX, 'seed');
    this.#state = value;
  }

  get state() {
    return this.#state;
  }

  bounded(upperBound) {
    const bound = typeof upperBound === 'bigint' ? upperBound : BigInt(upperBound);
    requireBigIntRange(bound, 1n, UINT32_MAX, 'upperBound');
    this.#state = (this.#state * RANDOM_MULTIPLIER + 1n) % UINT32_MODULUS;
    return (this.#state * bound) >> 32n;
  }

  random64() {
    const high = this.bounded(0x3fffffffn);
    const low = this.bounded(0xffffffffn);
    return (high << 32n) | low;
  }

  random64Below(upperBound) {
    const bound = typeof upperBound === 'bigint' ? upperBound : BigInt(upperBound);
    requireBigIntRange(bound, 1n, INT64_MAX, 'upperBound');
    return this.random64() % bound;
  }
}

export function pascalDiv(dividend, divisor) {
  const left = typeof dividend === 'bigint' ? dividend : BigInt(dividend);
  const right = typeof divisor === 'bigint' ? divisor : BigInt(divisor);
  if (right === 0n) throw new RangeError('divisor must not be zero');
  return left / right;
}

export function roundHalfEven(value) {
  if (!Number.isFinite(value)) throw new RangeError('value must be finite');
  const lower = Math.floor(value);
  const fraction = value - lower;
  if (fraction < 0.5) return lower;
  if (fraction > 0.5) return lower + 1;
  return lower % 2 === 0 ? lower : lower + 1;
}

export function levelUpTime(level) {
  requireInteger(level, 'level');
  if (level < 0 || level > 1_000) {
    throw new RangeError('level is outside the supported reference range');
  }
  const rawSeconds = (20 + (1.15 ** level)) * 60;
  return Object.freeze({
    rawSeconds,
    roundedSeconds: roundHalfEven(rawSeconds),
  });
}

export function weightedStatIndex(stats, random) {
  if (!Array.isArray(stats) || stats.length < 6) {
    throw new RangeError('stats must contain the six desktop prime stats');
  }
  const values = stats.slice(0, 6).map((value, index) => {
    const integer = typeof value === 'bigint' ? value : BigInt(value);
    requireBigIntRange(integer, 0n, INT64_MAX, `stats[${index}]`);
    return integer;
  });
  if (!(random instanceof DelphiRandomReference)) {
    throw new TypeError('random must be a DelphiRandomReference');
  }

  const weights = values.map((value) => value * value);
  const total = weights.reduce((sum, weight) => sum + weight, 0n);
  requireBigIntRange(total, 1n, INT64_MAX, 'weighted stat total');

  if (random.bounded(2n) < 1n) {
    return Number(random.bounded(BigInt(stats.length)));
  }

  let target = random.random64Below(total);
  for (let index = 0; index < weights.length; index += 1) {
    target -= weights[index];
    if (target < 0n) return index;
  }
  throw new Error('weighted stat selection exhausted its range');
}

export function randomNumericVectors() {
  const random = new DelphiRandomReference(0x12345678n);
  const bounded = [2n, 10n, 0xffffffffn].map((upperBound) => ({
    upperBound: upperBound.toString(),
    result: random.bounded(upperBound).toString(),
  }));
  const random64 = random.random64().toString();
  const random64Below = random.random64Below(1_000_003n).toString();
  const finalSeed = random.state.toString();

  const weightedRandom = new DelphiRandomReference(0x0badf00dn);
  const weightedStats = [12, 11, 10, 9, 8, 7, 1_000];
  const weightedIndex = weightedStatIndex(weightedStats, weightedRandom);

  return {
    format: 'gyrognome-desktop-random-numeric-vectors/v1',
    source: {
      tag: DESKTOP_SOURCE.tag,
      commit: DESKTOP_SOURCE.commit,
      mainPasSha256: DESKTOP_SOURCE.files['Main.pas'],
    },
    reference: {
      implementation: 'scripts/desktop-reference.mjs#random-numeric/v1',
      assumptions: NUMERIC_ASSUMPTIONS,
    },
    runtimeEquivalence: {
      status: 'unverified',
      officialExecutableCorroboration: 'not-run',
      localContinuationOnly: true,
      classicOnlineEligible: false,
    },
    vectors: {
      random: {
        initialSeed: '305419896',
        bounded,
        random64,
        random64Below: {
          upperBound: '1000003',
          result: random64Below,
        },
        finalSeed,
      },
      weightedStat: {
        initialSeed: '195948557',
        stats: weightedStats,
        selectedIndex: weightedIndex,
        finalSeed: weightedRandom.state.toString(),
      },
      integerDivision: [
        { dividend: '5114', divisor: '1000', result: '5' },
        { dividend: '-5114', divisor: '1000', result: '-5' },
      ],
      halfEvenRound: [
        { input: 1_200.5, result: 1_200 },
        { input: 1_201.5, result: 1_202 },
        { input: -1_200.5, result: -1_200 },
        { input: -1_201.5, result: -1_202 },
      ],
      levelUpTime: [1, 2, 10, 20, 30, 40].map((level) => ({
        level,
        ...levelUpTime(level),
      })),
      limits: {
        seed: { minimum: '0', maximum: UINT32_MAX.toString() },
        boundedUpper: { minimum: '1', maximum: UINT32_MAX.toString() },
        random64Below: { minimum: '1', maximum: INT64_MAX.toString() },
        weightedTotalMaximum: INT64_MAX.toString(),
      },
    },
  };
}

const PROLOGUE_NARRATIVE_QUEUE = Object.freeze([
  'task|10|Experiencing an enigmatic and foreboding night vision',
  "task|6|Much is revealed about that wise old bastard you'd underestimated",
  'task|6|A shocking series of events leaves you alone and bewildered, but resolute',
  'task|4|Drawing upon an unexpected reserve of determination, you set out on a long and dangerous journey',
]);

export function freshPrologue(layout) {
  if (!['6.2', '6.4.4'].includes(layout)) {
    throw new Error('unsupported desktop prologue layout');
  }
  const modern = layout === '6.4.4';
  return {
    layout,
    internalTask: modern ? 'load' : '',
    activity: modern ? 'Loading...' : 'Loading....',
    taskBar: { position: 0, max: 2_000 },
    queue: [
      ...PROLOGUE_NARRATIVE_QUEUE,
      modern ? 'plot|2|Loading' : 'task|2|Loading',
    ],
    questMarker: '',
    questTag: 0,
    plots: [{ caption: 'Prologue', completed: false }],
  };
}

export function applyDesktopLoadDefaults(serialized = {}) {
  return {
    spells: serialized.spells ?? [],
    quests: serialized.quests ?? [],
    taskBarPosition: serialized.taskBarPosition ?? 0,
    questBarPosition: serialized.questBarPosition ?? 0,
    plotBarPosition: serialized.plotBarPosition ?? 0,
    motto: serialized.motto ?? '',
    guild: serialized.guild ?? '',
  };
}

export function applySpellingPatch(spells) {
  if (!Array.isArray(spells)) throw new TypeError('spells must be an array');
  return spells.map((spell, index) => {
    if (typeof spell !== 'string') throw new TypeError('spell names must be strings');
    if (index === 0) return spell;
    if (spell === 'Innoculate') return 'Inoculate';
    if (spell === 'Tonsilectomy') return 'Tonsillectomy';
    return spell;
  });
}

function sha256(value) {
  return createHash('sha256').update(JSON.stringify(value)).digest('hex');
}

function spellingNormalizationRequests(spells) {
  const base = {
    traits: [
      { caption: 'Name', value: 'Synthetic Spelling Hero' },
      { caption: 'Race', value: 'Human' },
      { caption: 'Class', value: 'Fighter' },
      { caption: 'Level', value: '2' },
    ],
    experiencePosition: 42,
    equipment: { index: 0, value: 'Stick', caption: 'Weapon' },
    spells,
    stats: [
      { caption: 'STR', value: 12 },
      { caption: 'CON', value: 11 },
      { caption: 'DEX', value: 10 },
      { caption: 'INT', value: 9 },
      { caption: 'WIS', value: 8 },
      { caption: 'CHA', value: 7 },
    ],
    plots: ['Prologue', 'Act I'],
    realm: 'Synthetic Realm',
    motto: 'Synthetic motto',
    authentication: { credentialsPresent: true },
  };
  const salt = 4_242;
  return {
    automaticLevel: desktopReportReference({ ...base, trigger: 'l' }, salt),
    automaticAct: desktopReportReference({ ...base, trigger: 'a' }, salt),
    manualBrag: desktopReportReference({ ...base, trigger: 'b' }, salt),
    motto: desktopReportReference({
      ...base,
      trigger: 'm',
      motto: 'Changed synthetic motto',
    }, salt),
    guild: desktopGuildReference({
      traits: base.traits,
      realm: base.realm,
      guild: 'Synthetic Guild',
      authentication: base.authentication,
    }, salt),
  };
}

export function spellingNormalizationComparison() {
  const preCorrectionSpells = [
    { name: 'Gyp', rank: 'I' },
    { name: 'Innoculate', rank: 'II' },
    { name: 'Tonsilectomy', rank: 'III' },
  ];
  const alreadyCanonicalSpells = [
    { name: 'Gyp', rank: 'I' },
    { name: 'Inoculate', rank: 'II' },
    { name: 'Tonsillectomy', rank: 'III' },
  ];
  const normalizedSpells = preCorrectionSpells.map((spell, index) => ({
    ...spell,
    name: applySpellingPatch(preCorrectionSpells.map(({ name }) => name))[index],
  }));
  return {
    preCorrection: {
      serializedSpells: preCorrectionSpells,
      recordedAdaptations: ['load-spelling-patch'],
      postLoadCanonicalState: { spells: normalizedSpells },
      unsignedRequests: spellingNormalizationRequests(normalizedSpells),
    },
    alreadyCanonical: {
      serializedSpells: alreadyCanonicalSpells,
      recordedAdaptations: [],
      postLoadCanonicalState: { spells: alreadyCanonicalSpells },
      unsignedRequests: spellingNormalizationRequests(alreadyCanonicalSpells),
    },
  };
}

export function spellingNormalizationVectors() {
  const comparison = spellingNormalizationComparison();
  const summarize = (value) => ({
    serializedSpells: value.serializedSpells,
    recordedAdaptations: value.recordedAdaptations,
    postLoadCanonicalStateSha256: sha256(value.postLoadCanonicalState),
    unsignedRequestSha256: Object.fromEntries(
      Object.entries(value.unsignedRequests).map(
        ([operation, request]) => [operation, sha256(request)],
      ),
    ),
  });
  return {
    format: 'gyrognome-desktop-spelling-normalization-vectors/v1',
    source: {
      desktop644: {
        tag: DESKTOP_SOURCE.tag,
        commit: DESKTOP_SOURCE.commit,
        mainPasSha256: DESKTOP_SOURCE.files['Main.pas'],
      },
      spellingPatch: DESKTOP_LOAD_SOURCES.spellingPatch,
    },
    reference: {
      implementation: 'scripts/desktop-reference.mjs#spelling-normalization/v1',
      requestScope: ['automatic-level', 'automatic-act', 'manual-brag', 'motto', 'guild'],
    },
    cases: {
      preCorrection: summarize(comparison.preCorrection),
      alreadyCanonical: summarize(comparison.alreadyCanonical),
    },
  };
}

function parseQueueCommand(command) {
  if (typeof command !== 'string') throw new TypeError('queue command must be a string');
  const [kind, durationSeconds, ...captionParts] = command.split('|');
  const duration = Number(durationSeconds);
  if (!['task', 'plot'].includes(kind)
    || !Number.isSafeInteger(duration)
    || duration <= 0
    || captionParts.length === 0) {
    throw new Error('unsupported desktop queue command');
  }
  return { kind, durationMs: duration * 1_000, caption: captionParts.join('|') };
}

export function dequeueReference(state) {
  if (!Array.isArray(state?.queue) || state.queue.length === 0) {
    throw new Error('desktop reference queue is empty');
  }
  const command = parseQueueCommand(state.queue[0]);
  const next = structuredClone(state);
  next.queue.shift();
  next.internalTask = '';
  let actCompleted = false;

  if (command.kind === 'plot') {
    if (!Array.isArray(next.plots) || next.plots.length === 0) {
      throw new Error('plot transition requires an active plot');
    }
    next.plots.at(-1).completed = true;
    const actNumber = next.plots.length;
    next.plots.push({ caption: `Act ${roman(actNumber)}`, completed: false });
    next.plotBar = {
      position: 0,
      max: 60 * 60 * (1 + (5 * actNumber)),
      hint: 'Cutscene omitted',
    };
    next.activity = `Loading ${next.plots.at(-1).caption}...`;
    actCompleted = true;
  } else {
    next.activity = `${command.caption}...`;
  }
  next.taskBar = { position: 0, max: command.durationMs };
  return { state: next, actCompleted };
}

function roman(value) {
  if (value === 1) return 'I';
  if (value === 2) return 'II';
  if (value === 3) return 'III';
  throw new RangeError('synthetic reference supports acts I through III');
}

export function questMonsterFromPlaceholder({ questMarker, questTag, monsters }) {
  if (questMarker === '') return null;
  requireInteger(questTag, 'questTag');
  if (!Array.isArray(monsters) || questTag < 0 || questTag >= monsters.length) {
    throw new RangeError('questTag is outside the monster table');
  }
  return monsters[questTag];
}

export function loadingAdaptationVectors() {
  const legacy = freshPrologue('6.2');
  const modern = freshPrologue('6.4.4');
  const legacyFinal = dequeueReference({ ...legacy, queue: [legacy.queue.at(-1)] });
  const modernFinal = dequeueReference({ ...modern, queue: [modern.queue.at(-1)] });
  const spells = ['Innoculate', 'Innoculate', 'Tonsilectomy', 'Gyp', 'Shoelaces'];
  const monsters = ['Synthetic Beast|3|token', 'Synthetic Wyrm|4|scale'];

  return {
    format: 'gyrognome-desktop-loading-adaptation-vectors/v1',
    source: {
      desktop644: {
        tag: DESKTOP_SOURCE.tag,
        commit: DESKTOP_SOURCE.commit,
        mainPasSha256: DESKTOP_SOURCE.files['Main.pas'],
      },
      legacy62: DESKTOP_LOAD_SOURCES.legacy62,
      spellingPatch: DESKTOP_LOAD_SOURCES.spellingPatch,
    },
    runtimeEquivalence: {
      status: 'unverified',
      officialExecutableCorroboration: 'not-run',
      localContinuationOnly: true,
      classicOnlineEligible: false,
    },
    observations: {
      omittedDefaults: applyDesktopLoadDefaults(),
      prologue62: {
        loaded: legacy,
        finalQueueTransition: legacyFinal,
      },
      prologue644: {
        loaded: modern,
        finalQueueTransition: modernFinal,
      },
      questPlaceholder: {
        marker: 'fQuest',
        tag: 1,
        monsters,
        selectedMonster: questMonsterFromPlaceholder({
          questMarker: 'fQuest',
          questTag: 1,
          monsters,
        }),
      },
      spellPatch: {
        before: spells,
        after: applySpellingPatch(spells),
        firstRowPatched: false,
      },
    },
  };
}

function pick(values, random) {
  if (!Array.isArray(values) || values.length === 0) {
    throw new RangeError('reference table must not be empty');
  }
  return values[Number(random.bounded(BigInt(values.length)))];
}

function splitRule(value) {
  const separator = value.lastIndexOf('|');
  if (separator < 0) throw new Error('reference rule must contain a numeric suffix');
  const quality = Number(value.slice(separator + 1));
  if (!Number.isSafeInteger(quality)) throw new Error('reference rule quality is invalid');
  return { name: value.slice(0, separator), quality };
}

function randomLow(upperBound, random) {
  const bound = BigInt(upperBound);
  return Number(
    [random.bounded(bound), random.bounded(bound)].reduce(
      (minimum, value) => value < minimum ? value : minimum,
    ),
  );
}

function winSpellReference(state, random, spells) {
  const upperBound = Math.min(state.stats.WIS + state.level, spells.length);
  const name = spells[randomLow(upperBound, random)];
  const existing = state.spells.find((spell) => spell.name === name);
  if (existing) existing.rank += 1;
  else state.spells.push({ name, rank: 1 });
  return name;
}

function winStatReference(state, random) {
  const names = Object.keys(state.stats);
  const index = weightedStatIndex(names.map((name) => state.stats[name]), random);
  state.stats[names[index]] += 1;
  return names[index];
}

function closestRule(values, goal, random) {
  let result = pick(values, random);
  for (let count = 0; count < 5; count += 1) {
    const candidate = pick(values, random);
    if (Math.abs(goal - splitRule(result).quality)
      > Math.abs(goal - splitRule(candidate).quality)) {
      result = candidate;
    }
  }
  return result;
}

function winItemReference(state, random, tables) {
  const roll = Number(random.bounded(999n));
  if (Math.max(250, roll) < state.inventory.length) {
    const existing = state.inventory[Number(random.bounded(BigInt(state.inventory.length)))];
    existing.quantity += 1;
    return existing.name;
  }
  const name = `${pick(tables.itemAttrib, random)} ${pick(tables.specials, random)} of ${
    pick(tables.itemOf, random)
  }`;
  state.inventory.push({ name, quantity: 1 });
  return name;
}

function winEquipReference(state, random, tables) {
  const position = Number(random.bounded(BigInt(state.equipment.length)));
  const weapon = position === 0;
  const rules = weapon ? tables.weapons : position === 1 ? tables.shields : tables.armors;
  const better = weapon ? tables.offenseAttrib : tables.defenseAttrib;
  const worse = weapon ? tables.offenseBad : tables.defenseBad;
  const selected = splitRule(closestRule(rules, state.level, random));
  let name = selected.name;
  let plus = state.level - selected.quality;
  const modifiers = plus < 0 ? worse : better;
  for (let count = 0; count < 2 && plus !== 0; count += 1) {
    const modifier = splitRule(pick(modifiers, random));
    if (name.includes(modifier.name) || Math.abs(plus) < Math.abs(modifier.quality)) break;
    name = `${modifier.name} ${name}`;
    plus -= modifier.quality;
  }
  if (plus !== 0) name = `${plus > 0 ? '+' : ''}${plus} ${name}`;
  state.equipment[position] = name;
  state.prizedEquipment = position;
  return { position, name };
}

export function levelUpReference(input, random, tables) {
  const state = structuredClone(input);
  const events = [];
  state.level += 1;
  events.push({ type: 'level', value: state.level });
  const hp = Number(pascalDiv(BigInt(state.stats.CON), 3n)) + 1
    + Number(random.bounded(4n));
  state.stats['HP Max'] += hp;
  events.push({ type: 'stat', name: 'HP Max', amount: hp });
  const mp = Number(pascalDiv(BigInt(state.stats.INT), 3n)) + 1
    + Number(random.bounded(4n));
  state.stats['MP Max'] += mp;
  events.push({ type: 'stat', name: 'MP Max', amount: mp });
  for (let count = 0; count < 2; count += 1) {
    events.push({ type: 'stat', name: winStatReference(state, random), amount: 1 });
  }
  events.push({ type: 'spell', name: winSpellReference(state, random, tables.spells) });
  state.experience = { position: 0, max: levelUpTime(state.level).roundedSeconds };
  events.push({ type: 'experience-reset', ...state.experience });
  events.push({ type: 'report', trigger: 'l' });
  return { state, events, randomSeed: random.state.toString() };
}

export function completeActReference(input, random, tables) {
  const state = structuredClone(input);
  const events = [];
  state.plots.at(-1).completed = true;
  events.push({ type: 'plot-completed', caption: state.plots.at(-1).caption });
  const actNumber = state.plots.length;
  const caption = `Act ${roman(actNumber)}`;
  state.plotBar = {
    position: 0,
    max: 60 * 60 * (1 + (5 * actNumber)),
    hint: 'Cutscene omitted',
  };
  state.plots.push({ caption, completed: false });
  events.push({ type: 'plot-added', caption });
  if (state.plots.length > 2) {
    events.push({ type: 'item', name: winItemReference(state, random, tables) });
  }
  if (state.plots.length > 3) {
    events.push({ type: 'equipment', ...winEquipReference(state, random, tables) });
  }
  events.push({ type: 'report', trigger: 'a' });
  return { state, events, randomSeed: random.state.toString() };
}

export function completionCreditReference(input) {
  const credit = Number(pascalDiv(BigInt(input.taskMaxMs), 1_000n));
  const gain = input.internalTask.startsWith('kill|');
  return {
    credit,
    experience: input.experience + (gain ? credit : 0),
    quest: input.quest + (gain && input.hasQuest ? credit : 0),
    plot: Math.min(
      input.plotMax,
      input.plot + (input.internalTask !== 'load' ? credit : 0),
    ),
  };
}

const FORM_UNESCAPED = /^[A-Za-z0-9\-_.!~*'()]$/;

export function desktopFormEncode(value) {
  if (typeof value !== 'string') throw new TypeError('value must be a string');
  let encoded = '';
  for (const character of value) {
    const byte = character.codePointAt(0);
    if (byte > 0x7f) {
      throw new RangeError('desktop protocol values must be ASCII');
    }
    if (FORM_UNESCAPED.test(character)) encoded += character;
    else if (character === ' ') encoded += '+';
    else encoded += `%${byte.toString(16).toUpperCase().padStart(2, '0')}`;
  }
  return encoded;
}

export function desktopLfsr(plaintext, salt) {
  if (typeof plaintext !== 'string') throw new TypeError('plaintext must be a string');
  requireInteger(salt, 'salt');
  if (salt < -0x80000000 || salt > 0x7fffffff) {
    throw new RangeError('salt must be a signed 32-bit integer');
  }
  let result = salt | 0;
  for (const character of plaintext) {
    const byte = character.codePointAt(0);
    if (byte > 0x7f) {
      throw new RangeError('desktop validator input must be ASCII');
    }
    const feedback = ((result >> 31) ^ (result >> 5)) & 1;
    result = ((result << 1) ^ feedback ^ byte) | 0;
  }
  for (let count = 0; count < 10; count += 1) {
    const feedback = ((result >> 31) ^ (result >> 5)) & 1;
    result = ((result << 1) ^ feedback) | 0;
  }
  return result;
}

function romanToInteger(value) {
  const digits = { I: 1, V: 5, X: 10, L: 50, C: 100, D: 500, M: 1_000 };
  if (typeof value !== 'string' || value.length === 0 || !/^[IVXLCDM]+$/.test(value)) {
    throw new RangeError('spell rank must be a non-empty Roman numeral');
  }
  let result = 0;
  for (let index = 0; index < value.length; index += 1) {
    const current = digits[value[index]];
    const next = digits[value[index + 1]] ?? 0;
    result += current < next ? -current : current;
  }
  return result;
}

function desktopTraitFields(traits) {
  if (!Array.isArray(traits) || traits.length === 0) {
    throw new RangeError('traits must preserve desktop list order');
  }
  return traits.map(({ caption, value }, index) => {
    if (typeof caption !== 'string' || caption.length === 0 || typeof value !== 'string') {
      throw new TypeError(`traits[${index}] must contain string caption and value fields`);
    }
    return { name: caption[0].toLowerCase(), value };
  });
}

function encodedField(field) {
  return { ...field, encoded: desktopFormEncode(field.value) };
}

function reportEquipmentField(equipment) {
  if (typeof equipment?.value !== 'string' || typeof equipment?.caption !== 'string') {
    throw new TypeError('equipment must contain string value and caption fields');
  }
  requireInteger(equipment.index, 'equipment.index');
  const encoded = desktopFormEncode(equipment.value)
    + (equipment.index > 1 ? `+${desktopFormEncode(equipment.caption)}` : '');
  return { name: 'i', value: equipment.index > 1
    ? `${equipment.value} ${equipment.caption}`
    : equipment.value, encoded };
}

function bestSpellField(spells) {
  if (!Array.isArray(spells)) throw new TypeError('spells must be an array');
  if (spells.length === 0) return null;
  let best = 0;
  for (let index = 1; index < spells.length; index += 1) {
    const score = (index + 1) * romanToInteger(spells[index].rank);
    const bestScore = (best + 1) * romanToInteger(spells[best].rank);
    if (score > bestScore) best = index;
  }
  return encodedField({
    name: 'z',
    value: `${spells[best].name} ${spells[best].rank}`,
  });
}

function bestStatField(stats) {
  if (!Array.isArray(stats) || stats.length < 6) {
    throw new RangeError('stats must contain the six desktop prime stats');
  }
  for (let index = 0; index < 6; index += 1) {
    if (typeof stats[index]?.caption !== 'string') {
      throw new TypeError(`stats[${index}].caption must be a string`);
    }
    requireInteger(stats[index].value, `stats[${index}].value`);
  }
  let best = 0;
  for (let index = 1; index < 6; index += 1) {
    if (stats[index].value > stats[best].value) best = index;
  }
  return {
    name: 'k',
    value: `${stats[best].caption} ${stats[best].value}`,
    encoded: `${desktopFormEncode(stats[best].caption)}+${stats[best].value}`,
  };
}

function redactedAuthentication(authentication) {
  return {
    credentialsPresent: Boolean(authentication?.credentialsPresent),
    representation: authentication?.credentialsPresent ? 'legacy-url-userinfo' : 'none',
    retainedSeparately: true,
    preview: 'redacted',
  };
}

export function desktopReportReference(input, syntheticSalt) {
  const fieldsBeforeValidator = [
    encodedField({ name: 'cmd', value: 'b' }),
    encodedField({ name: 't', value: input.trigger }),
    ...desktopTraitFields(input.traits).map(encodedField),
    encodedField({ name: 'x', value: String(input.experiencePosition) }),
    reportEquipmentField(input.equipment),
  ];
  const spell = bestSpellField(input.spells);
  if (spell) fieldsBeforeValidator.push(spell);
  fieldsBeforeValidator.push(
    bestStatField(input.stats),
    encodedField({ name: 'a', value: input.plots.at(-1) }),
    encodedField({ name: 'h', value: input.realm }),
    encodedField({ name: 'rev', value: DESKTOP_PROTOCOL_SOURCE.revision }),
  );
  const queryBeforeValidator = fieldsBeforeValidator
    .map(({ name, encoded }) => `${name}=${encoded}`)
    .join('&');
  return {
    fieldsBeforeValidator,
    queryBeforeValidator,
    syntheticValidator: desktopLfsr(queryBeforeValidator, syntheticSalt),
    fieldsAfterValidator: [encodedField({ name: 'm', value: input.motto })],
    authentication: redactedAuthentication(input.authentication),
  };
}

export function desktopGuildReference(input, syntheticSalt) {
  const fieldsBeforeValidator = [
    encodedField({ name: 'cmd', value: 'guild' }),
    ...desktopTraitFields(input.traits).map(encodedField),
    encodedField({ name: 'h', value: input.realm }),
    encodedField({ name: 'rev', value: DESKTOP_PROTOCOL_SOURCE.revision }),
    encodedField({ name: 'guild', value: input.guild }),
  ];
  const queryBeforeValidator = fieldsBeforeValidator
    .map(({ name, encoded }) => `${name}=${encoded}`)
    .join('&');
  return {
    fieldsBeforeValidator,
    queryBeforeValidator,
    syntheticValidator: desktopLfsr(queryBeforeValidator, syntheticSalt),
    authentication: redactedAuthentication(input.authentication),
  };
}

export function protocolVectors() {
  const traits = [
    { caption: 'Name', value: 'Synthetic Hero & Co.' },
    { caption: 'Race', value: 'Half Orc' },
    { caption: 'Class', value: 'Robot Monk' },
    { caption: 'Level', value: '2' },
  ];
  const stats = [
    { caption: 'STR', value: 12 },
    { caption: 'CON', value: 11 },
    { caption: 'DEX', value: 10 },
    { caption: 'INT', value: 12 },
    { caption: 'WIS', value: 8 },
    { caption: 'CHA', value: 7 },
  ];
  const base = {
    traits,
    experiencePosition: 42,
    equipment: { index: 2, value: 'Banded Buckler', caption: 'Shield' },
    spells: [],
    stats,
    plots: ['Prologue', 'Act I'],
    realm: 'Synthetic Realm',
    motto: 'Ready & waiting',
    authentication: { credentialsPresent: true },
  };
  const salt = 42_424;
  const reports = {
    manual: desktopReportReference({ ...base, trigger: 'b' }, salt),
    level: desktopReportReference({
      ...base,
      trigger: 'l',
      experiencePosition: 0,
      spells: [{ name: 'Slime Finger', rank: 'I' }, { name: 'Rabbit Punch', rank: 'II' }],
    }, salt),
    act: desktopReportReference({
      ...base,
      trigger: 'a',
      plots: [...base.plots, 'Act II'],
      equipment: { index: 1, value: 'Polished Plate', caption: 'Shield' },
    }, salt),
    motto: desktopReportReference({
      ...base,
      trigger: 'm',
      motto: 'Changed motto!',
    }, salt),
  };
  const guild = desktopGuildReference({
    traits,
    realm: base.realm,
    guild: 'The A&B Guild',
    authentication: base.authentication,
  }, salt);
  return {
    format: 'gyrognome-desktop-protocol-vectors/v1',
    source: {
      tag: DESKTOP_SOURCE.tag,
      commit: DESKTOP_SOURCE.commit,
      mainPasSha256: DESKTOP_PROTOCOL_SOURCE.mainPasSha256,
      newGuyPasSha256: DESKTOP_PROTOCOL_SOURCE.newGuyPasSha256,
    },
    reference: {
      implementation: 'scripts/desktop-reference.mjs#protocol/v1',
      revision: DESKTOP_PROTOCOL_SOURCE.revision,
      encoding: DESKTOP_PROTOCOL_SOURCE.supportedEncoding,
    },
    limitations: {
      officialExecutableCorroboration: 'not-run',
      localContinuationOnly: true,
      classicOnlineEligible: false,
      fullSignedUrlsStored: false,
    },
    observations: { reports, guild },
  };
}

export function officialRuntimeCorroborationRecord() {
  return {
    format: 'gyrognome-desktop-official-runtime-corroboration/v1',
    recordedOn: DESKTOP_SOURCE.capturedOn,
    source: {
      tag: DESKTOP_SOURCE.tag,
      commit: DESKTOP_SOURCE.commit,
      mainPasSha256: DESKTOP_SOURCE.files['Main.pas'],
      officialExecutableSha256: DESKTOP_SOURCE.officialExecutableSha256,
    },
    decision: {
      status: 'not-performed',
      reason: 'separate approval and a suitable isolated Delphi-compatible runtime were unavailable',
      suppliedBinaryLaunched: false,
      exactDelphiCompilerRuntimeEquivalence: 'unverified',
      officialRuntimeClaim: false,
      localContinuationAllowed: true,
      classicOnlineDeliveryEnabled: false,
    },
    retainedEvidence: {
      sanitizedProvenanceOnly: true,
      runtimeObservations: [],
      networkObservations: [],
    },
  };
}

const REWARD_TABLES = Object.freeze({
  spells: Object.freeze(['Slime Finger', 'Rabbit Punch', 'Gyp', 'Shoelaces']),
  itemAttrib: Object.freeze(['Shiny', 'Ancient']),
  specials: Object.freeze(['Orb', 'Tome']),
  itemOf: Object.freeze(['Testing', 'Continuity']),
  weapons: Object.freeze(['Stick|0', 'Sword|5']),
  shields: Object.freeze(['Plate|1', 'Buckler|3']),
  armors: Object.freeze(['Burlap|3', 'Mail|8']),
  offenseAttrib: Object.freeze(['Polished|+1', 'Pronged|+2']),
  defenseAttrib: Object.freeze(['Studded|+1', 'Banded|+2']),
  offenseBad: Object.freeze(['Dull|-1', 'Bent|-2']),
  defenseBad: Object.freeze(['Cracked|-1', 'Rusty|-2']),
});

export function callbackRewardVectors() {
  let task = { position: 0, max: 6_000 };
  let afterSixty;
  for (let callback = 1; callback <= 60; callback += 1) {
    const result = timerCallback(task, 100);
    task = { ...task, position: result.position };
    if (callback === 60) afterSixty = result;
  }
  const callbackSixtyOne = timerCallback(task, 100);

  const baseLevel = {
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
  };
  const levelUp = levelUpReference(
    baseLevel,
    new DelphiRandomReference(0x13579bdfn),
    REWARD_TABLES,
  );

  const actBase = {
    level: 8,
    plots: [{ caption: 'Prologue', completed: true }, { caption: 'Act I', completed: false }],
    plotBar: { position: 1, max: 1 },
    inventory: [{ name: 'Gold', quantity: 0 }],
    equipment: ['Stick', 'Plate', 'Burlap'],
    prizedEquipment: 0,
  };
  const actTwo = completeActReference(
    actBase,
    new DelphiRandomReference(0x2468ace0n),
    REWARD_TABLES,
  );
  const actThree = completeActReference(
    {
      ...structuredClone(actTwo.state),
      plotBar: { position: actTwo.state.plotBar.max, max: actTwo.state.plotBar.max },
    },
    new DelphiRandomReference(BigInt(actTwo.randomSeed)),
    REWARD_TABLES,
  );

  return {
    format: 'gyrognome-desktop-callback-reward-vectors/v1',
    source: {
      tag: DESKTOP_SOURCE.tag,
      commit: DESKTOP_SOURCE.commit,
      mainPasSha256: DESKTOP_SOURCE.files['Main.pas'],
    },
    runtimeEquivalence: {
      status: 'unverified',
      officialExecutableCorroboration: 'not-run',
      localContinuationOnly: true,
      classicOnlineEligible: false,
    },
    observations: {
      callbacks: {
        initialTask: { position: 0, max: 6_000 },
        clamping: [
          { elapsedMs: -50, result: timerCallback({ position: 0, max: 6_000 }, -50) },
          { elapsedMs: 0, result: timerCallback({ position: 0, max: 6_000 }, 0) },
          { elapsedMs: 10_000, result: timerCallback({ position: 0, max: 6_000 }, 10_000) },
        ],
        fullBarSequence: {
          elapsedMsPerCallback: 100,
          advancementCallbacks: 60,
          afterSixty,
          callbackSixtyOne,
        },
      },
      fractionalCredits: [5_114, 5_508, 6_295].map((taskMaxMs) => ({
        taskMaxMs,
        kill: completionCreditReference({
          taskMaxMs,
          internalTask: 'kill|Synthetic Beast|1|token',
          experience: 0,
          quest: 0,
          hasQuest: true,
          plot: 0,
          plotMax: 100,
        }),
        market: completionCreditReference({
          taskMaxMs,
          internalTask: 'market',
          experience: 0,
          quest: 0,
          hasQuest: true,
          plot: 0,
          plotMax: 100,
        }),
      })),
      levelUp,
      actTwo,
      actThree,
    },
  };
}

export function timerCallback(task, elapsedMs) {
  requireInteger(task?.position, 'task.position');
  requireInteger(task?.max, 'task.max');
  requireInteger(elapsedMs, 'elapsedMs');
  if (task.max <= 0) throw new RangeError('task.max must be positive');
  if (task.position < 0 || task.position > task.max) {
    throw new RangeError('task.position must be within the task bar');
  }

  if (task.position >= task.max) {
    return Object.freeze({
      position: task.position,
      creditedMs: 0,
      completionDispatched: true,
    });
  }

  const creditedMs = Math.max(0, Math.min(elapsedMs, MAX_CALLBACK_ELAPSED_MS));
  return Object.freeze({
    position: Math.min(task.position + creditedMs, task.max),
    creditedMs,
    completionDispatched: false,
  });
}

export function syntheticSmoke() {
  const inputs = Object.freeze([250, 75]);
  let task = Object.freeze({ position: 5_900, max: 6_000 });
  const observations = inputs.map((elapsedMs) => {
    const result = timerCallback(task, elapsedMs);
    task = Object.freeze({ ...task, position: result.position });
    return Object.freeze({ elapsedMs, ...result });
  });

  return {
    format: 'gyrognome-desktop-reference-smoke/v1',
    source: DESKTOP_SOURCE,
    scenario: {
      id: 'fill-then-dispatch',
      initialTask: { position: 5_900, max: 6_000 },
      callbacks: observations,
      finalTask: task,
    },
  };
}

function exactKeys(value, keys) {
  return value !== null
    && typeof value === 'object'
    && !Array.isArray(value)
    && Object.keys(value).length === keys.length
    && keys.every((key) => Object.hasOwn(value, key));
}

function isSha256(value) {
  return typeof value === 'string' && /^[0-9a-f]{64}$/.test(value);
}

function validImplementationIdentity(value) {
  return exactKeys(value, ['implementation', 'contentSha256'])
    && typeof value.implementation === 'string'
    && value.implementation.length > 0
    && isSha256(value.contentSha256);
}

function containsSensitiveEvidence(value, key = '') {
  const normalizedKey = key.replaceAll(/[-_]/g, '').toLowerCase();
  if ([
    'account',
    'accountlogin',
    'credential',
    'credentials',
    'login',
    'passkey',
    'password',
    'rawresponse',
    'response',
    'responsebody',
    'rawsave',
    'savebytes',
    'signedurl',
    'authenticatedurl',
  ].includes(normalizedKey)) {
    return true;
  }
  if (typeof value === 'string') {
    try {
      const url = new URL(value);
      if (url.username || url.password) return true;
      for (const queryKey of url.searchParams.keys()) {
        if (['p', 'passkey', 'password', 'account', 'login'].includes(queryKey.toLowerCase())) {
          return true;
        }
      }
    } catch {
      // Non-URL evidence strings are inspected by key only.
    }
    return false;
  }
  if (Array.isArray(value)) {
    return value.some((child) => containsSensitiveEvidence(child));
  }
  if (value !== null && typeof value === 'object') {
    return Object.entries(value).some(
      ([childKey, child]) => containsSensitiveEvidence(child, childKey),
    );
  }
  return false;
}

export function validateDesktopEvidence(evidence) {
  if (!exactKeys(
    evidence,
    ['format', 'source', 'reference', 'production', 'runtimeEquivalence', 'observations'],
  )) {
    throw new Error('desktop evidence shape is incomplete');
  }
  if (containsSensitiveEvidence(evidence)) {
    throw new Error('desktop evidence contains sensitive or raw data');
  }
  if (evidence.format !== DESKTOP_EVIDENCE_FORMAT) {
    throw new Error('desktop evidence format is unsupported');
  }
  if (!exactKeys(
    evidence.source,
    [
      'repository',
      'tag',
      'commit',
      'capturedOn',
      'buildToolchain',
      'files',
      'officialExecutableSha256',
      'officialExecutableCorroboration',
    ],
  ) || evidence.source.repository !== DESKTOP_SOURCE.repository
    || evidence.source.tag !== DESKTOP_SOURCE.tag
    || evidence.source.commit !== DESKTOP_SOURCE.commit
    || !isSha256(evidence.source.officialExecutableSha256)
    || !exactKeys(evidence.source.files, Object.keys(DESKTOP_SOURCE.files))
    || !Object.values(evidence.source.files).every(isSha256)) {
    throw new Error('desktop source identity is incomplete');
  }
  if (!validImplementationIdentity(evidence.reference)
    || !validImplementationIdentity(evidence.production)) {
    throw new Error('desktop implementation identity is incomplete');
  }
  if (!exactKeys(
    evidence.runtimeEquivalence,
    [
      'status',
      'officialExecutableCorroboration',
      'localContinuationOnly',
      'classicOnlineEligible',
    ],
  )) {
    throw new Error('desktop runtime-equivalence limitation is undisclosed');
  }
  const runtime = evidence.runtimeEquivalence;
  if (!['unverified', 'corroborated'].includes(runtime.status)
    || !['not-run', 'performed'].includes(runtime.officialExecutableCorroboration)
    || typeof runtime.localContinuationOnly !== 'boolean'
    || typeof runtime.classicOnlineEligible !== 'boolean'
    || (runtime.status === 'unverified'
      && (runtime.localContinuationOnly !== true || runtime.classicOnlineEligible !== false))
    || (runtime.status === 'corroborated'
      && runtime.officialExecutableCorroboration !== 'performed')) {
    throw new Error('desktop runtime-equivalence limitation is invalid');
  }
  if (!Array.isArray(evidence.observations) || evidence.observations.length === 0) {
    throw new Error('desktop evidence observations are incomplete');
  }
  for (const observation of evidence.observations) {
    if (!exactKeys(observation, ['id', 'input', 'output'])
      || typeof observation.id !== 'string'
      || observation.id.length === 0
      || observation.input === undefined
      || observation.output === undefined) {
      throw new Error('desktop evidence observation is incomplete');
    }
  }
  return true;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  if (process.argv.length !== 3 || process.argv[2] !== '--smoke') {
    console.error('usage: node scripts/desktop-reference.mjs --smoke');
    process.exitCode = 1;
  } else {
    process.stdout.write(`${JSON.stringify(syntheticSmoke(), null, 2)}\n`);
  }
}
