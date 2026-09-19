# Spec Delta

## MODIFIED Requirements

### Requirement: Local-only runtime boundary

The runtime SHALL store and operate character data locally, except that an
active online managed character SHALL make one best-effort request to the
official leaderboard endpoint for each persisted browser-equivalent level-up or
act-completion report event. It SHALL persist the corresponding canonical
state before attempting delivery, SHALL not queue or retry a failed automatic
delivery, and SHALL continue advancing after a delivery failure. Offline
characters SHALL never report. Commands and diagnostic output SHALL NOT expose
browser save passkeys or raw unrecognized save fields.

#### Scenario: Reporting persisted online progression

- **WHEN** an active online managed character persists an advancement that
  emits a level-up or act-completion report event
- **THEN** the runtime sends one corresponding browser-compatible report to
  the official leaderboard endpoint after the state is persisted

#### Scenario: Running a managed online character

- **WHEN** a user starts a locally managed character that originated from an
  online browser save
- **THEN** the runtime advances it locally, sends only its persisted level-up
  and act-completion reports, and does not display its passkey

#### Scenario: Running a managed offline character

- **WHEN** a user starts a locally managed character without an online
  credential
- **THEN** the runtime advances it locally without making a network request
  or displaying a passkey

#### Scenario: Failing automatic report delivery

- **WHEN** an automatic report cannot be delivered or is rejected by the
  endpoint
- **THEN** the runtime preserves the persisted progression, does not retry or
  queue the report, and continues later advancement

#### Scenario: Invoking an explicit report submission

- **WHEN** an operator invokes a supported foreground reporting action for an
  eligible managed character
- **THEN** that action may contact the official leaderboard endpoint without
  exposing credentials
