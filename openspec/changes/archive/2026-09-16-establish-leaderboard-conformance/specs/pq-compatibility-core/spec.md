## MODIFIED Requirements

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
