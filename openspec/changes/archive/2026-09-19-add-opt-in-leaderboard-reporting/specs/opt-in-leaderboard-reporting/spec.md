## Purpose

Allow an operator to deliberately submit the current state of a
browser-imported managed character while preserving Gyrognome's default
credential-safe, transport-free operation.

## ADDED Requirements

### Requirement: Confirmed one-shot managed-character reporting

The system SHALL provide a foreground command that submits one
browser-compatible manual-brag (`t=b`) report for a selected managed character.
The command SHALL display the credential-safe target identity and require an
explicit interactive confirmation for each submission. Declining confirmation
SHALL perform no network request and leave managed state unchanged.

#### Scenario: Confirming a report submission

- **WHEN** an operator confirms reporting for an eligible managed character
- **THEN** the system submits exactly one browser-compatible manual-brag report
  for its current persisted state

#### Scenario: Declining a report submission

- **WHEN** an operator declines the reporting confirmation
- **THEN** the system sends no request and leaves the managed character
  unchanged

### Requirement: Reporting eligibility and conformance gate

The system SHALL allow reporting only for a managed character imported from a
valid online browser save with an available passkey, when no local runtime owns
the character and the bundled enrollment-conformance evidence passes
validation. It SHALL reject offline-originated characters, missing or invalid
online credentials, active runtimes, and invalid or incomplete conformance
evidence before attempting a network request.

#### Scenario: Reporting an offline-originated character

- **WHEN** an operator requests reporting for a managed character without an
  imported online browser credential
- **THEN** the system reports that the character is ineligible and sends no
  request

#### Scenario: Reporting an active managed character

- **WHEN** an operator requests reporting for a character currently owned by a
  local runtime
- **THEN** the system refuses submission and preserves the active runtime and
  managed state

#### Scenario: Conformance evidence is unavailable

- **WHEN** enrollment-conformance evidence does not pass validation
- **THEN** the system refuses all report submissions without contacting the
  leaderboard

### Requirement: Credential-safe report delivery

The reporting command SHALL construct the browser-compatible request from the
selected character's current persisted canonical state and retained passkey,
and SHALL send it only to the official leaderboard endpoint. It SHALL expose
only the safe identity, request outcome, and safe failure category; it SHALL
NOT print, persist in diagnostics, or include in errors the passkey, complete
signed URL, raw save document, response body, or browser profile data. A
successful HTTP response SHALL be reported only as delivered and SHALL NOT be
treated as proof of server-side leaderboard classification.

#### Scenario: Delivering an eligible report

- **WHEN** the official endpoint accepts a report request with a successful
  HTTP status
- **THEN** the system reports a credential-free delivered outcome without
  claiming leaderboard classification

#### Scenario: Handling delivery failure

- **WHEN** the report request cannot be delivered or returns an unsuccessful
  HTTP status
- **THEN** the system reports a credential-free failure, preserves the managed
  character, and does not retry automatically

### Requirement: No implicit or automated reporting

The system SHALL NOT submit leaderboard reports during registration,
inspection, dashboard refresh, worker advancement, lifecycle operations, or
any command other than the explicitly confirmed reporting command.

#### Scenario: Advancing a managed character

- **WHEN** a managed character advances through its worker or local lifecycle
- **THEN** the system performs no leaderboard request
