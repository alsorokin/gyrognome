# opt-in-leaderboard-reporting Specification

## Purpose

Allow an operator to deliberately submit the current state of a
browser-imported managed character while preserving Gyrognome's default
credential-safe, transport-free operation.

## Requirements

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

The system SHALL allow a manual-brag report, motto action, or guild action for
a managed character imported from a valid online browser save with an
available passkey while its local runtime is active or inactive, provided the
applicable conformance evidence passes validation. It SHALL serialize each
explicit online action with other online actions for the same character and
SHALL NOT stop, restart, or interrupt an active runtime to perform the action.
It SHALL reject offline-originated characters, missing or invalid online
credentials, and invalid or incomplete evidence before attempting a network
request.

#### Scenario: Reporting an offline-originated character

- **WHEN** an operator requests a manual brag, motto change, or guild action
  for a managed character without an imported online browser credential
- **THEN** the system reports that the character is ineligible and sends no
  request

#### Scenario: Reporting an active managed character

- **WHEN** an operator requests a manual-brag report for an otherwise eligible
  character currently owned by a local runtime
- **THEN** the system submits exactly one report through the serialized online
  action boundary without interrupting runtime ownership
- **AND** the report uses a coherent current persisted character and profile
  snapshot

#### Scenario: Editing the profile of an active managed character

- **WHEN** an operator requests a motto or guild action for an otherwise
  eligible character currently owned by a local runtime
- **THEN** the system performs the action through the serialized online-action
  boundary without interrupting runtime ownership

#### Scenario: Conformance evidence is unavailable

- **WHEN** evidence required for the requested online operation does not pass
  validation
- **THEN** the system refuses that operation without contacting the
  leaderboard
### Requirement: Credential-safe report delivery

The reporting surface SHALL construct browser-compatible manual-brag,
motto-change, and guild requests from a coherent persisted character and
online profile snapshot plus the retained passkey, and SHALL send them only to
the official leaderboard endpoint. It SHALL expose only the safe identity,
requested operation, and categorized outcome; it SHALL NOT print, persist in
diagnostics, or include in errors the passkey, complete signed URL, raw save
document, raw response body, or browser profile data. A successful HTTP
response for a report SHALL be reported only as delivered and SHALL NOT be
treated as proof of server-side leaderboard classification. Guild responses SHALL be read only to a fixed maximum size, normalized by
removing the prior and submitted guild designations, and interpreted only by
matching credential-safe response fingerprints from the sanitized
browser-derived conformance evidence. Raw response bodies and normalized text
SHALL remain internal and SHALL NOT be logged, returned, or persisted.

#### Scenario: Delivering an eligible report

- **WHEN** the official endpoint accepts a manual-brag or motto-change request
  with a successful HTTP status
- **THEN** the system reports a credential-free delivered outcome without
  claiming leaderboard classification

#### Scenario: Categorizing a guild response

- **WHEN** the official endpoint returns a guild response matching a
  browser-derived accepted, rejected, or indeterminate form
- **THEN** the system exposes only the corresponding safe outcome category and
  never exposes the raw response

#### Scenario: Guild response fingerprint is unknown

- **WHEN** a bounded guild response does not match a validated accepted or
  rejected normalized fingerprint
- **THEN** the system reports an indeterminate outcome and does not expose or
  persist the response body or normalized text

#### Scenario: Handling delivery failure

- **WHEN** an online request cannot be delivered or returns an unsuccessful
  HTTP status
- **THEN** the system reports a credential-free failure, preserves all state
  not explicitly defined as locally persistent by the requested action, and
  does not retry automatically
### Requirement: No implicit or automated reporting

The system SHALL submit browser-compatible reports only for persisted level-up
and act-completion events from an active online managed-character runtime, or
for an operator's explicit manual-brag, motto-change, or guild-designation
submission. An empty guild designation represents leaving the current guild.
Automatic reports SHALL be one-attempt best-effort deliveries and SHALL not be
queued or retried. Registration, inspection, dashboard refresh, lifecycle
operations, and administration SHALL NOT independently submit leaderboard
requests.

#### Scenario: Advancing a managed online character

- **WHEN** an active managed online character persists an advancement that
  emits a level-up or act-completion report event
- **THEN** the runtime submits exactly one corresponding report without
  requiring foreground confirmation

#### Scenario: Advancing a managed character

- **WHEN** a managed character advances through its worker or local lifecycle
- **THEN** the system submits reports only for persisted level-up and
  act-completion events from an online worker

#### Scenario: Advancing a managed offline character

- **WHEN** an active managed character without an online credential advances
- **THEN** the runtime submits no leaderboard request

#### Scenario: Refreshing a dashboard

- **WHEN** the dashboard refreshes a managed character without the operator
  invoking an explicit online action
- **THEN** it sends no leaderboard request
