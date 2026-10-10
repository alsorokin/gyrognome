# Spec Delta

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
recently accessed character first. When an identifier is supplied, it
SHALL open that character directly. The dashboard SHALL display the persisted
credential-safe identity, current activity, progress bars, equipment,
inventory, spells, plot, and local runtime service status. In the full
dashboard layout, the Adventure content SHALL NOT include a separate plot
row or current-quest row, regardless of Journal collapse state. The Adventure
pane SHALL separate its inventory and spells lists with an empty line.
The dashboard SHALL present a separate Journal pane whose expanded title
appends the canonical current plot caption after `Journal - `, for example
`Journal - Act VIII`. When Journal is collapsed and the canonical current quest
is non-empty, its title SHALL append ` - <current quest>` after that plot
caption, for example `Journal - Act VIII - Fetch me an anvil`, without a
`Current quest:` label. When the current quest is empty, the collapsed title
SHALL remain `Journal - <plot caption>` without an extra separator.
The collapsed Journal SHALL retain its two-row allocation and right-aligned
F6 shortcut. Titles exceeding the available header width SHALL be clipped
within the header without wrapping or hiding F6.
The expanded Journal pane's first line SHALL be the bold current-quest value
without a label and its subsequent completed quests SHALL be ordered from most
recent to oldest. The collapsed title SHALL reflect the latest displayed
persisted quest on refresh. It SHALL report an actionable error when the
requested character is not registered.

#### Scenario: Opening a registered character

- **WHEN** a user opens the dashboard for a registered managed character
- **THEN** the system renders its current credential-safe state and runtime
  service status in the terminal

#### Scenario: Selecting a character before opening the dashboard

- **WHEN** a user invokes `gyro dashboard` without an identifier and
  selects a registered character from the presented list
- **THEN** the system opens the dashboard for that selected character

#### Scenario: Viewing character recency and activity before selection

- **WHEN** a user invokes `gyro dashboard` without an identifier and
  multiple managed characters are registered
- **THEN** the selection list displays each character's last accessed time and
  whether that character is currently active
- **AND** the characters are ordered from most recently accessed to least
  recently accessed

#### Scenario: Cancelling dashboard selection

- **WHEN** a user invokes `gyro dashboard` without an identifier and
  cancels the character-selection flow
- **THEN** the system exits without opening a dashboard or changing any
  character data

#### Scenario: Opening the dashboard with no registrations

- **WHEN** a user invokes `gyro dashboard` without an identifier and no
  managed characters are registered
- **THEN** the system reports that no managed characters are available and
  does not enter an interactive dashboard

#### Scenario: Opening an unknown character

- **WHEN** a user opens the dashboard with an identifier that is not registered
- **THEN** the system reports that the managed character was not found and does
  not enter the interactive terminal view

#### Scenario: Viewing plot context in the full dashboard

- **WHEN** a registered character's canonical current plot caption is `Act VIII`
  and Journal is expanded
- **THEN** the Journal pane title is `Journal - Act VIII`
- **AND** the Adventure content does not render a separate plot row

#### Scenario: Viewing quest information in the full dashboard

- **WHEN** a registered character has completed quests and an expanded Journal
  pane
- **THEN** the Journal displays the bold current-quest value without a label
  first and lists completed quests from most recent to oldest below it, while
  Adventure omits the duplicate current-quest line

#### Scenario: Viewing quest information with Journal collapsed

- **WHEN** a registered character's plot caption is `Act VIII`, its current
  quest is `Fetch me an anvil`, Journal is collapsed, and the header is wide
  enough for the title
- **THEN** the Journal title is `Journal - Act VIII - Fetch me an anvil`
- **AND** Adventure shows inventory and spells separated by one empty line,
  with no current-quest row or additional quest separator

#### Scenario: Expanding Journal again

- **WHEN** the user expands a collapsed Journal
- **THEN** the title returns to `Journal - <plot caption>` and the bold current
  quest and completed history appear in its content
- **AND** Adventure still omits the current-quest row

#### Scenario: Refreshing a collapsed Journal

- **WHEN** a persisted-state refresh changes the current quest while Journal is
  collapsed, including when Adventure is also collapsed
- **THEN** the Journal title reflects the newly displayed current quest

#### Scenario: Viewing an empty current quest

- **WHEN** Journal is collapsed and the canonical current quest is empty
- **THEN** its title contains only `Journal - <plot caption>` with no trailing
  quest separator

#### Scenario: Viewing a long collapsed title

- **WHEN** the collapsed Journal title exceeds the available header width
- **THEN** the title is clipped within its header while F6 remains visible
- **AND** Journal remains two rows high without wrapping or spilling into
  another pane
