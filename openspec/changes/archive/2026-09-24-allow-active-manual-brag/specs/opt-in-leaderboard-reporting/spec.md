## MODIFIED Requirements

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
