## Purpose

Establish evidence that Gyrognome's locally advanced characters produce
browser-equivalent leaderboard report histories before general reporting exists.

## ADDED Requirements

### Requirement: Browser-equivalent report traces

The system SHALL derive an ordered, credential-free leaderboard report trace
from deterministic character advancement and explicitly requested browser
actions. Each trace entry SHALL identify the browser trigger and the complete
canonical snapshot at the point that the browser emits that report. The trace
SHALL preserve the browser's event order when a single advancement crosses
multiple report-producing transitions.

#### Scenario: Capturing a level-up report snapshot

- **WHEN** an advancement completes a task that reaches a level boundary
- **THEN** the derived trace contains an `l` entry with the canonical snapshot
  after the browser's level-up effects and before later completion effects

#### Scenario: Capturing an act-completion report snapshot

- **WHEN** an advancement completes an act
- **THEN** the derived trace contains an `a` entry at the browser-equivalent
  act-completion point

#### Scenario: Capturing explicit browser actions

- **WHEN** a caller requests a manual brag or changes a motto in the conformance
  input
- **THEN** the derived trace contains respectively a `b` or `m` entry with the
  corresponding browser-equivalent canonical snapshot

### Requirement: Credential-free trace conformance

The system SHALL verify browser-derived report traces using disposable,
synthetic character observations. A trace fixture SHALL record event order,
trigger, canonical snapshot, expected unsigned request fields, normalized
request representation, and validator result using a synthetic passkey.
Committed fixtures and diagnostics SHALL NOT contain player saves, live
passkeys, browser profiles, or complete signed leaderboard URLs.

#### Scenario: Replaying a browser-derived trace

- **WHEN** the system replays a committed report-trace fixture
- **THEN** every derived event's trigger, snapshot, unsigned fields,
  normalized representation, and synthetic validator match the fixture exactly

#### Scenario: Rejecting sensitive trace data

- **WHEN** a report-trace fixture or diagnostic artifact contains a player save,
  live passkey, browser profile, or complete signed request URL
- **THEN** the system rejects it before it can be committed or displayed

### Requirement: Controlled anti-cheat conformance evidence

The project SHALL define a documented, explicitly confirmed Playwright-harness
procedure for external leaderboard conformance experiments. The official
browser client SHALL create the newly created, disposable online character and
its passkey SHALL remain only in the ephemeral browser experiment. The
procedure SHALL compare browser and Gyrognome report histories across initial
load, pause, restart, delayed callbacks, task completion, level-up, act
completion, manual bragging, and motto change. It SHALL record only
credential-free observations and SHALL require evidence that the Gyrognome
character appears in the normal leaderboard population rather than the cheater
population. Native `newguy` character-generation support is not required.

#### Scenario: Running an external conformance experiment

- **WHEN** an operator explicitly confirms a disposable-character experiment
- **THEN** the harness creates the character through the official browser,
  sends no reports for any existing managed character, and records
  credential-free comparison and leaderboard-classification evidence for the
  disposable character

#### Scenario: Evidence is incomplete or classified as cheating

- **WHEN** a required scenario lacks normal-leaderboard evidence or the
  disposable character is classified in the cheater population
- **THEN** the conformance gate fails and general leaderboard reporting remains
  unsupported
