## MODIFIED Requirements

### Requirement: Managed character dashboard

The system SHALL provide an interactive terminal dashboard for a specified
managed character. The dashboard SHALL display the persisted credential-safe
identity, current activity, progress bars, equipment, inventory, spells, plot,
and local runtime service status. In the full dashboard layout, the Adventure
pane SHALL show the adventure data and only the current quest, labeled `Current
quest:`, without bold styling. The dashboard SHALL present a separate Journal
pane whose first line is the bold current-quest value without a label and whose
subsequent completed quests are ordered from most recent to oldest. It SHALL
report an actionable error when the requested character is not registered.

#### Scenario: Opening a registered character

- **WHEN** a user opens the dashboard for a registered managed character
- **THEN** the system renders its current credential-safe state and runtime
  service status in the terminal

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

## ADDED Requirements

### Requirement: Collapsible dashboard panes

The full dashboard SHALL let the user independently collapse and expand the
Activity, Progress, Equipment, Details, Status, Adventure, and Journal
panes using F1 through F7, respectively. Each of those pane headers SHALL
display its assigned hotkey right-aligned. Collapsing a pane SHALL hide its
content and reduce its layout allocation while preserving the visibility and
state of every other pane. When both are expanded, Journal SHALL consume no
more than one quarter of the usable right-column height and Adventure SHALL
receive the remaining space. When expanded in the full dashboard layout,
Details SHALL occupy exactly five terminal rows, consisting of two border rows
and three inner-content rows. When expanded in the full dashboard layout,
Progress SHALL occupy exactly seven terminal rows, consisting of two border
rows and five inner progress-bar rows. The dashboard SHALL retain its existing
compact view for terminals that do not use the full layout.

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
