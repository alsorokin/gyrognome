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
dashboard layout, the Adventure content SHALL NOT include a separate plot row.
The Adventure pane SHALL separate its inventory and spells lists with an empty
line. It SHALL show the non-bold current quest labeled `Current quest:` only
when the Journal pane is collapsed, separated from the spells list by an empty
line. The dashboard SHALL present a separate
Journal pane whose title appends the canonical current plot caption after
`Journal - `, for example `Journal - Act VIII`. The Journal pane's first line
SHALL be the bold current-quest value without a label and its subsequent
completed quests SHALL be ordered from most recent to oldest. It SHALL report
an actionable error when the requested character is not registered.

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
- **THEN** the Journal pane title is `Journal - Act VIII`
- **AND** the Adventure content does not render a separate plot row

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
- **AND** an empty line separates it from the spells list

### Requirement: Collapsible dashboard panes

The full dashboard SHALL let the user independently collapse and expand the
Activity, Progress, Equipment, Details, Adventure, and Journal panes using F1
through F6, respectively. Each collapsible pane header SHALL display its
assigned hotkey right-aligned. The Status pane SHALL remain expanded and SHALL
NOT have a collapse hotkey. The full dashboard SHALL place Keys and Status in
the same bottom row, with Keys on the left and Status on the right. Keys SHALL
use the same one-third width as the full dashboard's left pane column, and
Status SHALL use the remaining two-thirds width. In the normal full-dashboard
state, each bottom pane SHALL occupy three terminal rows when the complete Keys
line fits in one inner-content row. When the Keys line does not fit, both bottom
panes SHALL expand to four terminal rows so Keys can wrap across two
inner-content rows.
The Keys pane SHALL render each shortcut key with bold emphasis.
Confirmation warnings MAY temporarily increase the bottom-row height so their
safety text remains visible. Collapsing a pane SHALL hide its content and reduce
its layout allocation while preserving the visibility and state of every other
pane. When both are expanded, Journal
SHALL consume no more than one quarter of the usable right-column height and
Adventure SHALL receive the remaining space. When expanded in the full
dashboard layout, Activity SHALL occupy exactly four terminal rows, consisting
of two border rows and two inner-content rows. Expanded Details SHALL fill all
remaining left-column height after allocating Activity, Progress, and the
capped Equipment pane, with at least eight total rows when space permits.
Collapsed Details SHALL occupy only its two border rows. Progress SHALL occupy
exactly seven terminal rows, consisting of two border rows and five inner
progress-bar rows. Equipment SHALL receive flexible space when available but
its inner content height SHALL NOT exceed eleven rows, matching the maximum
number of equipment slots. The dashboard SHALL retain its existing compact
character and Status view for terminals that do not use the full layout. The
compact Keys pane SHALL use one inner-content row when its complete shortcut
line fits and SHALL expand to two inner-content rows only when that line wraps.

#### Scenario: Collapsing a pane

- **WHEN** a user presses the F-key assigned to a visible collapsible
  full-layout pane
- **THEN** the dashboard hides that pane's content, reduces its layout
  allocation, and keeps the other panes visible

#### Scenario: Restoring a collapsed pane

- **WHEN** a user presses the F-key assigned to a collapsed pane
- **THEN** the dashboard restores the pane's content and layout allocation

#### Scenario: Keeping Status visible

- **WHEN** the full dashboard is displayed or another pane is collapsed
- **THEN** Status remains expanded in the bottom-right pane without an F-key
  shortcut

#### Scenario: Arranging the bottom panes

- **WHEN** the full dashboard layout is displayed
- **THEN** Keys and Status share the bottom row with Keys on the left and
  Status on the right
- **AND** Keys uses one third of the width and Status uses two thirds
- **AND** Keys displays every normal shortcut/action group separated by ` | `,
  wrapping only when necessary
- **AND** each pane has one inner-content row when the Keys line fits and two
  inner-content rows when it wraps
- **AND** the Keys pane renders each shortcut key in bold

#### Scenario: Allocating Journal height

- **WHEN** Adventure and Journal are both expanded in the full dashboard layout
- **THEN** Journal receives no more than one quarter of the usable right-column
  height and Adventure receives the remaining space

#### Scenario: Allocating Activity height

- **WHEN** Activity is expanded in the full dashboard layout
- **THEN** it occupies exactly four terminal rows with two border rows and two
  inner-content rows

#### Scenario: Allocating Details height

- **WHEN** Details is expanded in the full dashboard layout
- **THEN** it fills all remaining left-column height after allocating the other
  panes, reserving at least eight total rows when space permits
- **AND** no unused vertical space remains below Details

#### Scenario: Collapsing Details

- **WHEN** Details is collapsed in the full dashboard layout
- **THEN** it occupies only two border rows rather than absorbing unused space

#### Scenario: Allocating Progress height

- **WHEN** Progress is expanded in the full dashboard layout
- **THEN** it occupies exactly seven terminal rows with two border rows and
  five inner progress-bar rows

#### Scenario: Limiting Equipment height

- **WHEN** Equipment is expanded and additional vertical space is available
- **THEN** its inner content area consumes no more than eleven rows

#### Scenario: Discovering pane shortcuts

- **WHEN** the full dashboard layout is displayed
- **THEN** Activity, Progress, Equipment, Details, Adventure, and Journal show
  F1, F2, F3, F4, F5, and F6 respectively on the right side of their headers

#### Scenario: Viewing the compact dashboard

- **WHEN** the terminal is below the full-layout width threshold
- **THEN** the dashboard continues to render its compact character and Status
  view
- **AND** its Keys pane uses one content row when the shortcut line fits and
  two only when it wraps
