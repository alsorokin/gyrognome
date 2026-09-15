# terminal-dashboard Specification

## Purpose

Provide an interactive, credential-safe terminal view of a locally managed
character and its runtime lifecycle without duplicating simulation behavior.

## Requirements

### Requirement: Managed character dashboard

The system SHALL provide an interactive terminal dashboard for a specified
managed character. The dashboard SHALL display the persisted credential-safe
identity, current activity, progress bars, equipment, inventory, spells, plot,
quests, and local runtime service status. It SHALL report an actionable error
when the requested character is not registered.

#### Scenario: Opening a registered character

- **WHEN** a user opens the dashboard for a registered managed character
- **THEN** the system renders its current credential-safe state and runtime
  service status in the terminal

#### Scenario: Opening an unknown character

- **WHEN** a user opens the dashboard with an identifier that is not registered
- **THEN** the system reports that the managed character was not found and does
  not enter the interactive terminal view

### Requirement: Live persisted-state refresh

The dashboard SHALL refresh its displayed managed-character state and runtime
status at a bounded periodic interval and when the user requests a refresh. It
SHALL read the persisted canonical state and service status only; it SHALL NOT
own a character lock, advance simulation, or write character state.

#### Scenario: Viewing an active runtime

- **WHEN** the character's local runtime persists new state while the dashboard
  is open
- **THEN** the dashboard displays the newer persisted state on a subsequent
  refresh without advancing the character itself

#### Scenario: Refreshing an inactive runtime

- **WHEN** the character runtime is inactive and the user requests a refresh
- **THEN** the dashboard continues to display the last persisted state and
  updated inactive or failed service status

### Requirement: Lifecycle controls

The dashboard SHALL expose keyboard actions to start, stop, and recover the
selected character's local runtime through the existing user-service lifecycle
interface. Before executing a lifecycle action, it SHALL request confirmation.
It SHALL report successful actions and actionable service-manager or runtime
failures in the dashboard without exiting.

#### Scenario: Starting an inactive runtime

- **WHEN** a user confirms the dashboard start action for an inactive character
- **THEN** the dashboard starts the character through the local user-service
  lifecycle interface and refreshes the displayed status

#### Scenario: Handling a lifecycle failure

- **WHEN** a confirmed lifecycle action fails because the service manager is
  unavailable or the runtime cannot start
- **THEN** the dashboard displays the error and preserves the most recently
  displayed credential-safe character state

### Requirement: Terminal-safe interaction

The dashboard SHALL provide visible keyboard help and let the user quit through
a documented key action or terminal interrupt. It SHALL restore terminal mode
and screen contents after normal exit, an input/rendering error, or an
interrupted lifecycle action.

#### Scenario: Quitting the dashboard

- **WHEN** a user invokes the dashboard quit action
- **THEN** the system restores the terminal and exits successfully

#### Scenario: Terminal failure

- **WHEN** terminal setup, input, or rendering fails after interactive mode was
  entered
- **THEN** the system restores the terminal before reporting the error

### Requirement: Credential-safe local-only presentation

The dashboard SHALL never display browser passkeys, raw browser save documents,
or unrecognized raw save fields. It SHALL NOT make HTTP requests, report
leaderboard progress, or expose an action that enables either behavior.

#### Scenario: Viewing an online-originated character

- **WHEN** a user opens the dashboard for a character imported from an online
  browser save
- **THEN** the dashboard shows only its credential-safe canonical state and
  local service status without exposing the passkey or making a network request
