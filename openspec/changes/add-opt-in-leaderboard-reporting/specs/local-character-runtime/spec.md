## MODIFIED Requirements

### Requirement: Local-only runtime boundary

The runtime SHALL store and operate character data locally and SHALL NOT
perform HTTP requests, leaderboard reporting, or terminal-dashboard rendering,
except for the explicitly confirmed foreground managed-character reporting
workflow. Commands and diagnostic output SHALL NOT expose browser save passkeys
or raw unrecognized save fields.

#### Scenario: Running a managed online character

- **WHEN** a user starts a locally managed character that originated from an
  online browser save
- **THEN** the runtime advances it locally without making a network request or
  displaying its passkey

#### Scenario: Invoking an explicit report submission

- **WHEN** an operator confirms the dedicated reporting workflow for an
  eligible, inactive managed character
- **THEN** only that foreground workflow may contact the official leaderboard
  endpoint while worker and other runtime paths remain transport-free
