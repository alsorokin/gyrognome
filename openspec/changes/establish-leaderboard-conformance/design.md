## Context

The simulator currently proves final canonical state against browser-derived
checkpoints, and the compatibility core constructs one-shot signed request
data. The browser client instead invokes reporting from within state
transitions: it reports on initial load (`s`), level-up (`l`), act completion
(`a`), a user-requested brag (`b`), and motto change (`m`). The server's
classification logic is not public, so local compatibility is necessary but
not sufficient evidence.

## Goals / Non-Goals

**Goals:**

- Preserve the browser's point-in-transition report snapshots and event order.
- Make report payload compatibility testable without live credentials or HTTP.
- Define a reproducible, explicitly authorized external experiment that gates
  future general reporting on observed non-cheater classification.

**Non-Goals:**

- Add background, automatic, or production leaderboard reporting.
- Transmit an existing managed character's passkey or create a normal user
  character online.
- Infer or reimplement undisclosed server anti-cheat logic.
- Change local worker cadence, no-catch-up behavior, or deterministic
  simulation outcomes.

## Decisions

### Represent reporting as pure transition events

Simulation will return or emit an ordered trace of report events alongside its
usual state result. Each event contains a trigger and a clone of canonical
state at the browser call site; it contains neither a passkey nor a URL.

This makes level and act reports distinguishable from the state after
subsequent completion effects. Reconstructing events from only the final state
was rejected because it cannot recover browser call ordering or payload timing.

### Keep report construction separate from event generation

The protocol layer will turn an event snapshot plus caller-supplied test
credentials into inspectable request components. It remains transport-free and
will be tested using synthetic passkeys and redacted representations.

Embedding signing or a passkey in the simulator was rejected because it would
break the simulation's pure, credential-free boundary and make safe fixture
validation harder.

### Treat manual reports as explicit conformance inputs

Initial load, manual brag, and motto change are not deterministic task
completions. The conformance interface will represent them as explicit actions
with their browser triggers, including the current motto at the report point.

Guessing automatic report cadence from worker persistence was rejected: the
browser only sends these reports at named call sites, and an invented cadence
could make otherwise correct characters look implausible.

### Gate external evidence behind disposable, opt-in experiments

The external procedure will use Playwright to create a fresh disposable
character through the official browser, retain its passkey only in the
ephemeral browser profile, require a distinct operator confirmation, run
bounded scenarios, and capture redacted evidence. It will compare browser and
Gyrognome event traces before checking normal leaderboard placement and absence
from the cheater population.

Testing a real managed character was rejected because its passkey is a bearer
credential and an incorrect submission could irreversibly affect its
leaderboard history. Reimplementing browser character creation was rejected:
the native `newguy` port is planned separately and is not prerequisite to this
gate. Pure local fixtures alone were rejected because they cannot reveal
undisclosed server-side classification rules.

## Risks / Trade-offs

- [The server classification is delayed, unavailable, or undocumented] →
  define polling bounds and record an inconclusive result as a failed gate, not
  a passing result.
- [Browser behavior changes after fixture capture] → pin the browser revision
  and source hash in trace fixtures and invalidate the gate when they differ.
- [Trace APIs accidentally expose credentials or signed URLs] → use
  credential-free event types, synthetic-only test inputs, and fixture-safety
  validation for every persisted observation.
- [A worker interval differs from browser callback behavior] → include
  delayed-callback and pause/restart cases while retaining the existing capped
  timing policy.

## Migration Plan

1. Add pure trace generation and verify it preserves current simulation
   results.
2. Add browser-derived trace fixtures and protocol conformance tests.
3. Add the explicit disposable-character experiment procedure and execute it
   for every required scenario.
4. Keep all reporting paths disabled unless the recorded evidence passes.

Rollback removes trace and experiment support; no database migration or user
character state conversion is required.
