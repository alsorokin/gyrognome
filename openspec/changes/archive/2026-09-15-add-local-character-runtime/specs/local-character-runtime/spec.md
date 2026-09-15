## Purpose

Provide durable, exclusively owned, local-only Progress Quest character
execution that safely connects a managed service lifecycle to deterministic
simulation.

## ADDED Requirements

### Requirement: Local character registration and persistence

The system SHALL import a valid browser save into a local managed-character
store and persist its canonical character state, original save document, and
runtime metadata under the invoking user's data directory. Each managed
character SHALL have a stable local identifier. Invalid imports SHALL leave no
partially registered character, and a failed state update SHALL retain the
previous complete persisted state.

#### Scenario: Registering a valid browser save

- **WHEN** a user registers a valid browser `.pqw` save
- **THEN** the system creates one managed character with a stable identifier
  and makes its credential-safe canonical state available for local inspection

#### Scenario: Rejecting an invalid browser save

- **WHEN** a user attempts to register malformed or invalid browser save data
- **THEN** the system reports the import error and does not create a managed
  character or modify an existing one

#### Scenario: Recovering after an interrupted update

- **WHEN** runtime persistence is interrupted while recording an advancement
- **THEN** the managed character remains readable at either its complete
  pre-advancement state or its complete post-advancement state

### Requirement: Controlled offline advancement

The system SHALL advance a running managed character using elapsed time
measured only by its active local runtime process and SHALL durably record each
successful resulting state. It SHALL invoke the deterministic simulation
contract with explicit elapsed durations. Time while the runtime is stopped
SHALL NOT be applied as catch-up advancement.

#### Scenario: Advancing while the runtime is active

- **WHEN** a managed character runtime remains active across one or more
  advancement intervals
- **THEN** the system persists each resulting canonical state and exposes the
  accumulated progression through local inspection

#### Scenario: Restarting a stopped runtime

- **WHEN** a managed character runtime is stopped and later started
- **THEN** the system resumes from the most recently persisted state without
  applying elapsed downtime

#### Scenario: Simulation cannot advance a state

- **WHEN** deterministic simulation reports an unsupported transition or other
  error
- **THEN** the runtime reports the failure, retains the last successful state,
  and does not fabricate an advancement result

### Requirement: Exclusive character ownership

The system SHALL ensure that no more than one local runtime process owns a
managed character at a time. A competing start attempt SHALL identify the
existing ownership and leave the running character unaffected. Ownership from
an unexpectedly terminated process SHALL cease without requiring manual
database repair.

#### Scenario: Starting an already-owned character

- **WHEN** a user starts a runtime for a character already owned by another
  live local process
- **THEN** the system refuses the second start and reports that the character
  is already running

#### Scenario: Recovering ownership after an unexpected exit

- **WHEN** the owning runtime process exits unexpectedly
- **THEN** a later start can acquire ownership and resume from the last
  successfully persisted state

### Requirement: User-service lifecycle controls

The system SHALL provide command-line operations to start, stop, inspect
status for, and recover a managed character's local runtime through the
invoking user's service manager. Lifecycle operations SHALL report whether the
character is active and SHALL return actionable errors when the requested
character or user service cannot be managed.

#### Scenario: Starting a registered character

- **WHEN** a user starts a registered, unowned character
- **THEN** the system starts its user-scoped runtime service and reports the
  character as active

#### Scenario: Stopping an active character

- **WHEN** a user stops an active managed character
- **THEN** the system terminates its user-scoped runtime service after the last
  completed state update and reports the character as inactive

#### Scenario: Inspecting lifecycle status

- **WHEN** a user requests a managed character's runtime status
- **THEN** the system reports its identifier, persisted character identity, and
  whether its local runtime service currently owns it

### Requirement: Local-only runtime boundary

The runtime SHALL store and operate character data locally and SHALL NOT
perform HTTP requests, leaderboard reporting, or terminal-dashboard rendering.
Commands and diagnostic output SHALL NOT expose browser save passkeys or raw
unrecognized save fields.

#### Scenario: Running a managed online character

- **WHEN** a user starts a locally managed character that originated from an
  online browser save
- **THEN** the runtime advances it locally without making a network request or
  displaying its passkey
