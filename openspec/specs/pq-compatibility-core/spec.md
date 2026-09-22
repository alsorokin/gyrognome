# pq-compatibility-core Specification

## Purpose

Provide a safe, deterministic compatibility boundary between browser Progress Quest
character data and a future Linux-native client without reporting game progress.

## Requirements

### Requirement: Browser save interchange

The system SHALL import a browser `.pqw` export by Base64-decoding and parsing
its JSON document. It SHALL expose a typed canonical character state
containing traits, initial and current deterministic random state, attributes,
active and elapsed task state, progress bars, equipment, inventory, spells,
plots, quests, online metadata, and optional motto and guild profile values.
Missing motto and guild fields SHALL be interpreted as empty values because
the browser omits them until first assigned. It SHALL validate the nested type
and required content of each canonical field with field-specific errors. It
SHALL export a compatible Base64-encoded JSON document and preserve
unrecognized imported JSON data when no canonical state has been changed.

#### Scenario: Importing a valid browser save

- **WHEN** a user supplies a well-formed `.pqw` export from the browser client
- **THEN** the system makes its complete typed canonical character and online
  profile state available for inspection

#### Scenario: Importing optional profile fields

- **WHEN** a browser save contains string-valued motto or guild fields
- **THEN** the system exposes those values as credential-safe online profile
  state

#### Scenario: Importing a save before profile fields were assigned

- **WHEN** a valid browser save omits motto or guild
- **THEN** the system imports it with an empty value for each omitted field

#### Scenario: Rejecting an invalid save without loss

- **WHEN** a user supplies text that is not a valid Base64-encoded JSON save or
  has an invalid required nested canonical field or an invalid present profile
  field
- **THEN** the system reports a field-specific import error and does not create
  or modify character state

#### Scenario: Round-tripping unmodified imported data

- **WHEN** the system exports a successfully imported save without modifying
  canonical state
- **THEN** re-importing the export retains the imported canonical state,
  online profile values, and unrecognized JSON data
### Requirement: Deterministic browser-compatible primitives

The system SHALL reproduce the browser client’s Alea random-number generator state
transitions and the integer primitives used to construct leaderboard validators.
Given a reference initial state and sequence of operations, it SHALL produce the
same reference random values and restored-state continuation.

#### Scenario: Reproducing a reference random sequence

- **WHEN** the system is initialized with a sanitized browser Alea state fixture
- **THEN** it produces the fixture’s expected sequence of bounded random values

#### Scenario: Restoring random state

- **WHEN** the system saves Alea state after a reference sequence and restores it in a new instance
- **THEN** subsequent generated values match the uninterrupted reference sequence
### Requirement: Offline construction of leaderboard requests

The system SHALL construct the creation, progress-report, guild, and motto request
data used by the browser client, including browser-compatible parameter encoding,
protocol revision, URL normalization, and LFSR validation. Given a
credential-free browser-report transition snapshot and a synthetic passkey, it
SHALL construct the trigger-specific progress-report payload in the exact field
order and at the exact state represented by that snapshot. This compatibility
core SHALL NOT transmit those requests or create characters on a leaderboard.

#### Scenario: Constructing a sanitized progress report

- **WHEN** a caller provides a canonical online character state and a synthetic passkey
- **THEN** the system produces the expected sanitized report URL and validator from the reference fixture

#### Scenario: Constructing a trigger-specific report

- **WHEN** a caller provides a credential-free browser-report transition
  snapshot, its trigger, and a synthetic passkey
- **THEN** the system produces the browser-equivalent unsigned fields,
  normalized request representation, and validator for that exact snapshot

#### Scenario: Preventing network activity

- **WHEN** a caller invokes save inspection or any request-construction operation
- **THEN** the system performs no network request
### Requirement: Safe character inspection and reference data

The system SHALL provide a read-only character inspection interface with a
human-readable character sheet and a machine-readable JSON representation of
canonical state. Both representations SHALL include traits, attributes, active
and elapsed task state, progress bars, equipment, inventory, spells, plots,
quests, online realm metadata, and credential-safe motto and guild values. The
interface SHALL redact online passkeys and omit unrecognized raw JSON data by
default. Committed reference fixtures and diagnostic output SHALL NOT contain
real passkeys, full signed request URLs, browser profiles, or player save
exports.

#### Scenario: Inspecting an online character safely

- **WHEN** a user inspects an imported online character with the default format
- **THEN** the character sheet identifies its realm and complete
  credential-safe state, including motto and guild when present, while
  redacting its passkey

#### Scenario: Producing machine-readable inspection output

- **WHEN** a user requests JSON inspection output for an imported character
- **THEN** the system produces valid JSON for its canonical and online profile
  state without an online passkey or unrecognized raw JSON fields

#### Scenario: Committing compatibility fixtures

- **WHEN** browser observations are converted into project fixtures
- **THEN** the fixtures use synthetic credentials and contain no
  player-identifying save data
