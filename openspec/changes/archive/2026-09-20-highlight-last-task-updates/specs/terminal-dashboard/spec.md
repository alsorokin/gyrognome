# Spec Delta

## ADDED Requirements

### Requirement: Recent task update indicator

The dashboard SHALL display character stats in its header alongside the
character identity, rather than in an additional pane or the Adventure pane.
When the header has sufficient width, stats SHALL occupy the right side of the
identity row. When it does not, stats SHALL occupy a second header row. When a
refreshed persisted state records one or more completed tasks since the
dashboard's previously displayed state, the dashboard SHALL visually highlight
each currently displayed stat, equipment slot, inventory item, and spell whose
value was added or changed during that observed completion interval. The
indicator SHALL remain visible during subsequent refreshes that do not record
another completed task. On the next observed completed task, the dashboard
SHALL clear the previous indicator and show only values changed in the new
completion interval.

#### Scenario: Highlighting values updated by a task

- **WHEN** a dashboard refresh observes that completed-task count increased and
  the refreshed state contains changed spell, inventory, stat, or equipment
  values
- **THEN** the dashboard renders those changed current values with the recent
  task update indicator

#### Scenario: Retaining an indicator between task completions

- **WHEN** a dashboard refresh does not observe an increase in completed-task
  count after a task's updated values were highlighted
- **THEN** the dashboard retains the existing recent task update indicator

#### Scenario: Replacing an indicator after the next task

- **WHEN** a dashboard refresh observes a later increase in completed-task
  count
- **THEN** the dashboard removes the prior indicator and highlights only the
  current values changed since the previously displayed state

#### Scenario: Displaying stats in the header

- **WHEN** the dashboard has sufficient width for the identity and stats
  content
- **THEN** it right-aligns stats on the identity header row and does not render
  stats in the Adventure pane

#### Scenario: Wrapping header stats at narrow widths

- **WHEN** the dashboard header is too narrow for its identity and stats
  content on one row
- **THEN** it renders stats on a second header row while retaining the recent
  task update indicator

## MODIFIED Requirements

### Requirement: Managed character dashboard

The system SHALL provide an interactive terminal dashboard for a specified
managed character. The `dashboard` command SHALL accept an optional managed
character identifier. When no identifier is supplied, it SHALL present a
credential-safe interactive list of registered characters and open the
dashboard for the user-selected character. When an identifier is supplied, it
SHALL open that character directly. The dashboard SHALL display the persisted
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

#### Scenario: Cancelling dashboard selection

- **WHEN** a user invokes `gyrognome dashboard` without an identifier and
  cancels the character-selection flow
- **THEN** the system exits without opening a dashboard or changing any
  character data

#### Scenario: Opening the dashboard with no registrations

- **WHEN** a user invokes `gyrognome dashboard` without an identifier and no
  managed characters are registered
- **THEN** the system reports that no managed characters are available and does
  not enter an interactive dashboard

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
