# terminal-dashboard Specification

## Purpose

Provide an interactive, credential-safe terminal view of a locally managed
character and its runtime lifecycle without duplicating simulation behavior.

## Requirements

### Requirement: Managed character dashboard

The system SHALL provide an interactive terminal dashboard for a specified
managed character. The `dashboard` command SHALL accept an optional managed
character identifier. When no identifier is supplied, it SHALL present a
credential-safe interactive list of registered characters and open the
dashboard for the user-selected character. When an identifier is supplied, it
SHALL open that character directly. The dashboard SHALL display the persisted
credential-safe identity, current activity, progress bars, equipment,
inventory, spells, plot, and local runtime service status. In the full
dashboard layout, the Adventure pane SHALL show the adventure data and only
the current quest, labeled `Current quest:`, without bold styling. The
dashboard SHALL present a separate Journal pane whose first line is the bold
current-quest value without a label and whose subsequent completed quests are
ordered from most recent to oldest. It SHALL report an actionable error when
the requested character is not registered.

#### Scenario: Opening a registered character

- **WHEN** a user opens the dashboard for a registered managed character
- **THEN** the system renders its current credential-safe state and runtime
  service status in the terminal

#### Scenario: Selecting a character before opening the dashboard

- **WHEN** a user invokes `gyrognome dashboard` without an identifier and
  selects a registered character from the presented list
- **THEN** the system opens the dashboard for that selected character

#### Scenario: Cancelling dashboard selection

- **WHEN** a user invokes `gyrognome dashboard` without an identifier and
  cancels the character-selection flow
- **THEN** the system exits without opening a dashboard or changing any
  character data

#### Scenario: Opening the dashboard with no registrations

- **WHEN** a user invokes `gyrognome dashboard` without an identifier and no
  managed characters are registered
- **THEN** the system reports that no managed characters are available and
  does not enter an interactive dashboard

#### Scenario: Opening an unknown character

- **WHEN** a user opens the dashboard with an identifier that is not registered
- **THEN** the system reports that the managed character was not found and does
  not enter the interactive terminal view

#### Scenario: Viewing quest information in the full dashboard

- **WHEN** a registered character has completed quests and a current quest
- **THEN** the Adventure pane displays only the non-bold current quest with the
  `Current quest:` label, while the Journal displays the bold current-quest
  value without a label first and lists completed quests from most recent to
  oldest below it

### Requirement: Collapsible dashboard panes

The full dashboard SHALL let the user independently collapse and expand the
Activity, Progress, Equipment, Details, Status, Adventure, and Journal panes
using F1 through F7, respectively. Each of those pane headers SHALL display
its assigned hotkey right-aligned. Collapsing a pane SHALL hide its content and
reduce its layout allocation while preserving the visibility and state of every
other pane. When both are expanded, Journal SHALL consume no more than one
quarter of the usable right-column height and Adventure SHALL receive the
remaining space. When expanded in the full dashboard layout, Details SHALL
occupy exactly five terminal rows, consisting of two border rows and three
inner-content rows. When expanded in the full dashboard layout, Progress SHALL
occupy exactly seven terminal rows, consisting of two border rows and five
inner progress-bar rows. The dashboard SHALL retain its existing compact view
for terminals that do not use the full layout.

#### Scenario: Collapsing a pane

- **WHEN** a user presses the F-key assigned to a visible full-layout pane
- **THEN** the dashboard hides that pane's content, reduces its layout
  allocation, and keeps the other panes visible

#### Scenario: Restoring a collapsed pane

- **WHEN** a user presses the F-key assigned to a collapsed pane
- **THEN** the dashboard restores the pane's content and layout allocation

#### Scenario: Allocating Journal height

- **WHEN** Adventure and Journal are both expanded in the full dashboard layout
- **THEN** Journal receives no more than one quarter of the usable right-column
  height and Adventure receives the remaining space

#### Scenario: Allocating Details height

- **WHEN** Details is expanded in the full dashboard layout
- **THEN** it occupies exactly five terminal rows with two border rows and
  three inner-content rows

#### Scenario: Allocating Progress height

- **WHEN** Progress is expanded in the full dashboard layout
- **THEN** it occupies exactly seven terminal rows with two border rows and
  five inner progress-bar rows

#### Scenario: Discovering pane shortcuts

- **WHEN** the full dashboard layout is displayed
- **THEN** each collapsible pane header shows its corresponding F1 through F7
  hotkey on the right side

#### Scenario: Viewing the compact dashboard

- **WHEN** the terminal is below the full-layout width threshold
- **THEN** the dashboard continues to render its compact character view

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
