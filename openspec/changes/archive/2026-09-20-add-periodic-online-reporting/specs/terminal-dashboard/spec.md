# Spec Delta

## MODIFIED Requirements

### Requirement: Credential-safe local-only presentation

The dashboard SHALL never display browser passkeys, raw browser save documents,
or unrecognized raw save fields. It SHALL provide an immediate manual-brag
action that sends one browser-compatible report for an eligible managed
character through the official leaderboard endpoint. It SHALL show only the
credential-safe delivery outcome or safe failure category. Dashboard refresh,
navigation, rendering, and lifecycle actions SHALL NOT independently make HTTP
requests or report leaderboard progress.

#### Scenario: Bragging from the dashboard

- **WHEN** an operator invokes the dashboard manual-brag action for an eligible
  managed character
- **THEN** the dashboard immediately sends one browser-compatible manual-brag
  report and displays a credential-safe delivery outcome

#### Scenario: Viewing an online-originated character

- **WHEN** a user opens the dashboard for a character imported from an online
  browser save
- **THEN** the dashboard shows only its credential-safe canonical state and
  local service status without exposing the passkey or making a network request
  until the operator invokes the manual-brag action

#### Scenario: Refreshing without bragging

- **WHEN** the dashboard refreshes, renders, navigates, or executes a lifecycle
  action without the operator invoking the manual-brag action
- **THEN** it sends no leaderboard request
