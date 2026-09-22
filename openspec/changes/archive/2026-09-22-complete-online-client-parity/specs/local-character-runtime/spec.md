# Spec Delta

## MODIFIED Requirements

### Requirement: Local character registration and persistence

The system SHALL import a valid browser save into a local managed-character
store and persist its canonical character state, credential-safe online
profile metadata, original save document, and runtime metadata under the
invoking user's data directory. Each managed character SHALL have a stable
local identifier. Profile metadata SHALL be independently updateable without
overwriting concurrent canonical simulation progress. Invalid imports SHALL
leave no partially registered character, and a failed state or profile update
SHALL retain the previous complete persisted value.

#### Scenario: Registering a valid browser save

- **WHEN** a user registers a valid browser `.pqw` save
- **THEN** the system creates one managed character with a stable identifier
  and makes its credential-safe canonical and online profile state available
  for local inspection

#### Scenario: Rejecting an invalid browser save

- **WHEN** a user attempts to register malformed or invalid browser save data
- **THEN** the system reports the import error and does not create a managed
  character or modify an existing one

#### Scenario: Recovering after an interrupted update

- **WHEN** runtime persistence is interrupted while recording an advancement
- **THEN** the managed character remains readable at either its complete
  pre-advancement state or its complete post-advancement state

#### Scenario: Recovering after an interrupted profile update

- **WHEN** persistence is interrupted while changing motto or guild metadata
- **THEN** the managed character retains either the complete previous profile
  or the complete requested profile without corrupting simulation state

### Requirement: Local-only runtime boundary

The runtime SHALL store and operate character data locally, except that an
active online managed character SHALL make one best-effort request to the
official leaderboard endpoint for each persisted browser-equivalent level-up
or act-completion report event. Each automatic report SHALL include the
character's currently persisted motto. The runtime SHALL persist the
corresponding canonical state before attempting delivery, SHALL serialize the
delivery with other online actions for that character, SHALL not queue or
retry a failed automatic delivery, and SHALL continue advancing after a
delivery failure. Offline characters SHALL never report. Commands and
diagnostic output SHALL NOT expose browser save passkeys or raw unrecognized
save fields.

#### Scenario: Reporting persisted online progression

- **WHEN** an active online managed character persists an advancement that
  emits a level-up or act-completion report event
- **THEN** the runtime sends one corresponding browser-compatible report with
  the current persisted motto to the official leaderboard endpoint after the
  state is persisted

#### Scenario: Changing a motto near an automatic report

- **WHEN** a foreground motto action and an automatic report contend for the
  same character's online-action boundary
- **THEN** their requests are sent in acquisition order and the later request
  uses the profile state established before its snapshot

#### Scenario: Running a managed online character

- **WHEN** a user starts a locally managed character that originated from an
  online browser save
- **THEN** the runtime advances it locally, sends only its persisted level-up
  and act-completion reports, and does not display its passkey

#### Scenario: Running a managed offline character

- **WHEN** a user starts a locally managed character without an online
  credential
- **THEN** the runtime advances it locally without making a network request or
  displaying a passkey

#### Scenario: Failing automatic report delivery

- **WHEN** an automatic report cannot be delivered or is rejected by the
  endpoint
- **THEN** the runtime preserves the persisted progression and profile,
  does not retry or queue the report, and continues later advancement

#### Scenario: Invoking an explicit report submission

- **WHEN** an operator invokes a supported foreground report or online profile
  action for an eligible managed character
- **THEN** that action may contact the official leaderboard endpoint without
  stopping an active runtime or exposing credentials
