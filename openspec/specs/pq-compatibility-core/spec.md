# pq-compatibility-core Specification

## Purpose

Provide a safe, deterministic compatibility boundary between browser Progress Quest
character data and a future Linux-native client without reporting game progress.

## Requirements

### Requirement: Browser save interchange

The system SHALL import a browser `.pqw` export by Base64-decoding and parsing its
JSON document. It SHALL expose the canonical character state needed by the browser
simulation, including traits, deterministic random state, progress, task state,
bars, inventory, spells, quests, and online metadata. It SHALL export a compatible
Base64-encoded JSON document and preserve unrecognized imported JSON data when no
canonical state has been changed.

#### Scenario: Importing a valid browser save

- **WHEN** a user supplies a well-formed `.pqw` export from the browser client
- **THEN** the system makes its canonical character state available for inspection

#### Scenario: Rejecting an invalid save without loss

- **WHEN** a user supplies text that is not a valid Base64-encoded JSON save
- **THEN** the system reports an import error and does not create or modify character state

#### Scenario: Round-tripping unmodified imported data

- **WHEN** the system exports a successfully imported save without modifying canonical state
- **THEN** re-importing the export retains the imported canonical state and unrecognized JSON data

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
protocol revision, URL normalization, and LFSR validation. This compatibility core
SHALL NOT transmit those requests or create characters on a leaderboard.

#### Scenario: Constructing a sanitized progress report

- **WHEN** a caller provides a canonical online character state and a synthetic passkey
- **THEN** the system produces the expected sanitized report URL and validator from the reference fixture

#### Scenario: Preventing network activity

- **WHEN** a caller invokes save inspection or any request-construction operation
- **THEN** the system performs no network request

### Requirement: Safe character inspection and reference data

The system SHALL provide a read-only character inspection interface that summarizes
canonical state without exposing an online passkey by default. Committed reference
fixtures and diagnostic output SHALL NOT contain real passkeys, full signed request
URLs, browser profiles, or player save exports.

#### Scenario: Inspecting an online character safely

- **WHEN** a user inspects an imported online character
- **THEN** the summary identifies its realm and online status while redacting its passkey

#### Scenario: Committing compatibility fixtures

- **WHEN** browser observations are converted into project fixtures
- **THEN** the fixtures use synthetic credentials and contain no player-identifying save data
