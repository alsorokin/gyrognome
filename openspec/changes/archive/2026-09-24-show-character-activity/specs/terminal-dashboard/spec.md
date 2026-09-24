## MODIFIED Requirements

### Requirement: Managed character dashboard

The system SHALL provide an interactive terminal dashboard for a specified
managed character. The `dashboard` command SHALL accept an optional managed
character identifier. When no identifier is supplied, it SHALL present a
credential-safe interactive list of registered characters and open the
dashboard for the user-selected character. The selection list SHALL display
each character's credential-safe identity, stable identifier, last accessed
time, and whether its local runtime is currently active. The selection list
SHALL order characters by last accessed time descending, with the most
recently accessed character first. When an identifier is supplied, it SHALL
open that character directly. The dashboard SHALL display the persisted
credential-safe identity, current activity, progress bars, equipment,
inventory, spells, plot, and local runtime service status. In the full
dashboard layout, the Adventure pane SHALL separate its inventory and spells
lists with an empty line. It SHALL show the non-bold current quest labeled
`Current quest:` only when the Journal pane is collapsed. The dashboard SHALL
present a separate Journal pane whose first line is the bold current-quest
value without a label and whose subsequent completed quests are ordered from
most recent to oldest. It SHALL report an actionable error when the requested
character is not registered.

#### Scenario: Opening a registered character

- **WHEN** a user opens the dashboard for a registered managed character
- **THEN** the system renders its current credential-safe state and runtime
  service status in the terminal

#### Scenario: Selecting a character before opening the dashboard

- **WHEN** a user invokes `gyrognome dashboard` without an identifier and
  selects a registered character from the presented list
- **THEN** the system opens the dashboard for that selected character

#### Scenario: Viewing character recency and activity before selection

- **WHEN** a user invokes `gyrognome dashboard` without an identifier and
  multiple managed characters are registered
- **THEN** the selection list displays each character's last accessed time and
  whether that character is currently active
- **AND** the characters are ordered from most recently accessed to least
  recently accessed

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

- **WHEN** a registered character has completed quests and an expanded Journal
  pane
- **THEN** the Journal displays the bold current-quest value without a label
  first and lists completed quests from most recent to oldest below it, while
  Adventure omits the duplicate current-quest line

#### Scenario: Viewing quest information with Journal collapsed

- **WHEN** a registered character has a current quest and the Journal pane is
  collapsed
- **THEN** Adventure displays only the non-bold current quest with the
  `Current quest:` label
