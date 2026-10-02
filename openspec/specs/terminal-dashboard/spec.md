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
dashboard for the user-selected character. The selection list SHALL display
each character's credential-safe identity, stable identifier, last accessed
time, and whether its local runtime is currently active. The selection list
SHALL order characters by last accessed time descending, with the most
recently accessed character first. When an identifier is supplied, it
SHALL open that character directly. The dashboard SHALL display the persisted
credential-safe identity, current activity, progress bars, equipment,
inventory, spells, plot, and local runtime service status. In the full
dashboard layout, the Adventure content SHALL NOT include a separate plot
row. The Adventure pane SHALL separate its inventory and spells lists with an
empty line. It SHALL show the non-bold current quest labeled `Current quest:`
only when the Journal pane is collapsed, separated from the spells list by an
empty line. The dashboard SHALL present a separate Journal pane whose title
appends the canonical current plot caption after `Journal - `, for example
`Journal - Act VIII`. The Journal pane's first line SHALL be the bold
current-quest value without a label and its subsequent completed quests SHALL
be ordered from most recent to oldest. It SHALL report an actionable error
when the requested character is not registered.

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

### Requirement: Recent task update indicator

The dashboard SHALL display character stats in its header alongside the
character identity, rather than in an additional pane or the Adventure pane.
Adjacent stats SHALL be separated by ` | `.
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

### Requirement: Collapsible dashboard panes

The full dashboard SHALL let the user independently collapse and expand the
Activity, Progress, Equipment, Details, Adventure, and Journal panes using F1
through F6, respectively. Each collapsible pane header SHALL display its
assigned hotkey right-aligned. The Status pane SHALL remain expanded and SHALL
NOT have a collapse hotkey. The full dashboard SHALL place Keys and Status in
the same bottom row, with Keys on the left and Status on the right. Keys SHALL
use the same one-third width as the full dashboard's left pane column, and
Status SHALL use the remaining two-thirds width. In the normal full-dashboard
state, each bottom pane SHALL occupy three terminal rows when the complete
Keys line fits in one inner-content row. When the Keys line does not fit, both
bottom panes SHALL expand to four terminal rows so Keys can wrap across two
inner-content rows.
The Keys pane SHALL render each shortcut key with bold emphasis.
Confirmation warnings MAY temporarily increase the bottom-row height so their
safety text remains visible. Collapsing a pane SHALL hide its content and
reduce its layout allocation while preserving the visibility and state of
every other pane. When both are expanded, Journal SHALL consume no more than
one quarter of the usable right-column height and Adventure SHALL receive the
remaining space. When expanded in the full dashboard layout, Activity SHALL
occupy exactly four terminal rows, consisting of two border rows and two
inner-content rows. Expanded Details SHALL fill all remaining left-column
height after allocating Activity, Progress, and the capped Equipment pane,
with at least eight total rows when space permits. Collapsed Details SHALL
occupy only its two border rows. Progress SHALL occupy exactly seven terminal
rows, consisting of two border rows and five inner progress-bar rows.
Equipment SHALL receive flexible space when available but its inner content
height SHALL NOT exceed eleven rows, matching the maximum number of equipment
slots. The dashboard SHALL retain its existing compact character and Status
view for terminals that do not use the full layout. The compact Keys pane
SHALL use one inner-content row when its complete shortcut line fits and SHALL
expand to two inner-content rows only when that line wraps.

#### Scenario: Collapsing a pane

- **WHEN** a user presses the F-key assigned to a visible full-layout pane
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

### Requirement: Predicted current-task progress

The dashboard SHALL display a predicted position for the current task progress
bar between persisted state reads, derived from monotonic time elapsed since it
observed the most recent persisted state. The prediction SHALL be computed from
the observed persisted task position and task duration alone; it SHALL NOT
invoke the simulation, acquire a character lock, or write character state.

The prediction SHALL be display-only. It SHALL NOT be persisted, included in
any leaderboard report, or used to determine task completion, the
recent-task-update indicator, lifecycle eligibility, or manual-brag
eligibility.

The prediction SHALL apply only to the task progress bar, in both the full
Progress pane and the compact progress summary. The experience, encumbrance,
plot, and quest bars, the completed-task count, the activity description, the
character stats, equipment, inventory, spells, plots, and quests SHALL change
only when a newer persisted state is read.

The predicted position SHALL NOT exceed the current task's duration. When the
predicted position reaches that duration, the dashboard SHALL display a full
task bar and SHALL hold it there until a newer persisted state is read.

The dashboard SHALL predict only while a local runtime owns the character. When
the character is not owned, or ownership cannot be determined, the dashboard
SHALL display the persisted task position unchanged.

When the dashboard reads a newer persisted state, it SHALL discard the previous
prediction and re-anchor to the newly observed state.

The dashboard SHALL schedule its redraws so that a predicted task bar advances
in whole-percent steps for tasks of any duration. When a redraw is delayed, it
SHALL display the currently predicted percentage rather than replaying the
percentages it did not draw.

#### Scenario: Advancing within a task

- **WHEN** a local runtime owns the character and monotonic time elapses
  between persisted state reads
- **THEN** the displayed task bar advances in whole-percent steps toward the
  current task's duration without the dashboard advancing the character itself

#### Scenario: Reaching the end of a task before new state is read

- **WHEN** the predicted position reaches the current task's duration and no
  newer persisted state has been read
- **THEN** the dashboard displays a full task bar and leaves the completed-task
  count, rewards, and activity description at their persisted values

#### Scenario: Re-anchoring on newer persisted state

- **WHEN** the dashboard reads a persisted state that differs from the state
  the current prediction was anchored to
- **THEN** it discards the prediction and displays the newly persisted task
  position, activity, and rewards

#### Scenario: Observing a character with no active runtime

- **WHEN** no local runtime owns the character, or ownership cannot be
  determined
- **THEN** the dashboard displays the persisted task position without
  predicting any advancement

#### Scenario: Predicting without affecting other displayed values

- **WHEN** the dashboard displays a predicted task position
- **THEN** the recent-task-update indicator, progress bars other than the task
  bar, and every other credential-safe value continue to reflect only
  persisted state

#### Scenario: Recovering from a delayed redraw

- **WHEN** a redraw occurs later than the whole-percent step it was scheduled
  for
- **THEN** the dashboard displays the currently predicted percentage instead of
  replaying the intermediate percentages

### Requirement: Live persisted-state refresh

The dashboard SHALL refresh its displayed managed-character state, online
profile metadata, and runtime status at bounded periodic intervals without
requiring a manual refresh action. It MAY read persisted character/profile state
and runtime service status on separate bounded schedules.

When a predicted task position reaches the current task's duration, the
dashboard SHALL wait a bounded settling interval before reading persisted
character state, so that the local runtime has an opportunity to commit the
completed task first. It SHALL separate successive reads triggered this way by
at least that settling interval, and SHALL NOT delay such a read beyond its
regular state-refresh interval.

Except when executing an explicit motto or guild action, the dashboard SHALL
only read persisted character/profile state and service status; it SHALL NOT
own the simulation lock, advance simulation, or write canonical simulation
state. Explicit profile actions MAY write profile metadata through the
online-profile interface while leaving simulation ownership unchanged.

#### Scenario: Viewing an active runtime

- **WHEN** the character's local runtime persists new state while the dashboard
  is open
- **THEN** the dashboard displays the newer persisted state on a subsequent
  refresh without advancing the character itself

#### Scenario: Refreshing profile metadata

- **WHEN** a motto or guild value changes while the dashboard is open
- **THEN** the dashboard displays the newer persisted profile value on a
  subsequent refresh

#### Scenario: Refreshing an inactive runtime

- **WHEN** the character runtime remains inactive while the dashboard is open
- **THEN** the dashboard continues to display the last persisted state and
  updates the displayed inactive or failed service status automatically

#### Scenario: Reading state and service status on separate schedules

- **WHEN** the dashboard reads persisted character and profile state on a
  shorter schedule than runtime service status
- **THEN** it continues to display the most recently read service status and
  does not query the service manager on the shorter schedule

#### Scenario: Reading state after a predicted task ends

- **WHEN** a predicted task position reaches the current task's duration
- **THEN** the dashboard waits a bounded settling interval, giving the local
  runtime an opportunity to commit the completed task, and then reads
  persisted character state without waiting for its next regularly scheduled
  read

#### Scenario: Observing the next task at its starting position

- **WHEN** a read triggered by a saturated prediction returns a newer task than
  the one the prediction was anchored to
- **THEN** the dashboard displays that task's persisted position, rewards, and
  activity description, and anchors its next prediction to them

#### Scenario: Waiting for a runtime that has not yet persisted a completion

- **WHEN** a read triggered by a saturated prediction returns the same task the
  prediction was already anchored to
- **THEN** the dashboard continues to display a full task bar and separates its
  next triggered read from the previous one by at least the settling interval

### Requirement: Lifecycle controls

The dashboard SHALL expose one contextual keyboard action through the existing
user-service lifecycle interface. The action SHALL start an inactive selected
character runtime, stop an active runtime, and recover a failed runtime by
clearing its failed state before starting it. The dashboard SHALL NOT expose a
separate recovery action. Before executing a lifecycle action, it SHALL request
confirmation. It SHALL report successful actions and actionable
service-manager or runtime failures in the dashboard without exiting.

#### Scenario: Starting an inactive runtime

- **WHEN** a user confirms the dashboard lifecycle action for an inactive
  character
- **THEN** the dashboard starts the character through the local user-service
  lifecycle interface and refreshes the displayed status

#### Scenario: Stopping an active runtime

- **WHEN** a user confirms the same dashboard lifecycle action for an active
  character
- **THEN** the dashboard stops the character through the local user-service
  lifecycle interface and refreshes the displayed status

#### Scenario: Recovering a failed runtime

- **WHEN** a user confirms the same dashboard lifecycle action for a failed
  character
- **THEN** the dashboard clears the service's failed state, starts the
  character through the local user-service lifecycle interface, and refreshes
  the displayed status

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

The dashboard SHALL never display passkeys, account logins/passwords, raw save
documents/properties, signed/authenticated endpoint URLs, raw responses, or
unrecognized raw fields. It SHALL provide immediate manual brag and editable
motto/guild actions for an eligible character through its approved official
endpoint. Editors SHALL show current persisted values, accept printable
non-control Unicode input, use Enter to submit, and Escape to cancel. Profile
encoding restrictions SHALL be enforced before persistence or transport.
Empty guild input SHALL request leaving; no separate leave action is needed.
Only safe categorized outcomes SHALL be shown. Refresh, navigation, rendering,
and lifecycle actions SHALL NOT independently issue HTTP requests.

#### Scenario: Bragging from the dashboard

- **WHEN** an operator invokes manual brag for an eligible inactive character
- **THEN** one profile-compatible manual report is immediately attempted and a
  safe delivery outcome is displayed

#### Scenario: Changing a motto from the dashboard

- **WHEN** an eligible character's valid motto edit is submitted or cleared
- **THEN** one motto action is performed, the displayed profile is refreshed,
  and a safe outcome is shown

#### Scenario: Submitting a guild designation from the dashboard

- **WHEN** valid empty or non-empty guild input is submitted for an eligible
  character
- **THEN** one guild action is performed, the displayed profile is refreshed,
  and a safe category is shown

#### Scenario: Cancelling a profile editor

- **WHEN** an operator presses Escape in a motto or guild editor
- **THEN** it closes without state changes or requests

#### Scenario: Viewing an online-originated character

- **WHEN** a character imported from an online browser or desktop save is opened
- **THEN** only safe canonical/profile state and local status are shown,
  without credentials or any request until an explicit online action

#### Scenario: Refreshing without bragging

- **WHEN** the dashboard refreshes, renders, navigates, or performs a lifecycle
  action without an explicit online action
- **THEN** it sends no leaderboard request

### Requirement: Concise profile and timing details

The full dashboard Details pane SHALL omit the Quest target line. It SHALL
display the character identifier, Last task elapsed, and Realm. It SHALL
display Motto and Guild lines independently only when the corresponding
persisted value is non-empty.

Last task elapsed SHALL be formatted as a compact decomposition into days,
hours, minutes, and seconds rather than as one raw count of seconds. Leading
zero units SHALL be omitted and a zero duration SHALL render as `0s`.

Confirmed profile and lifecycle action outcomes SHALL remain visible across
automatic refreshes for at least five seconds so an operator can read the
result.

#### Scenario: Viewing online profile details

- **WHEN** a managed character has both a motto and guild designation
- **THEN** Details displays ID, formatted Last task elapsed, Motto,
  Guild, and Realm without displaying Quest target

#### Scenario: Viewing one populated profile value

- **WHEN** a managed character has a non-empty motto and no guild designation
- **THEN** Details displays Motto and omits Guild and Quest target

#### Scenario: Viewing a character without profile values

- **WHEN** a managed character has neither a motto nor a guild designation
- **THEN** Details omits Motto, Guild, and Quest target

#### Scenario: Formatting elapsed time

- **WHEN** Last task elapsed is 90061 seconds
- **THEN** the dashboard displays `1d 1h 1m 1s` rather than `90061 seconds`

#### Scenario: Formatting zero elapsed time

- **WHEN** Last task elapsed is zero
- **THEN** the dashboard displays `0s`

### Requirement: Visible compatibility and eligibility

The dashboard SHALL display the persisted continuation profile and safe
online-operation eligibility, including the reason for classic conformance
gating or local-only provenance and the fresh-import recovery instruction.
It SHALL distinguish unavailable desktop history from counters measured since
import. Profile selection SHALL come from persisted state, not display labels.
Task-bar prediction SHALL NOT dispatch completion, create report eligibility,
or replace a pending desktop full-bar state with an assumed next task.

#### Scenario: Viewing a desktop local-only fork

- **WHEN** an online-originated desktop import has advanced while reporting
  was gated
- **THEN** the dashboard shows local-only status, disables online actions, and
  explains that later online use requires a fresh official-client import

#### Scenario: Rendering a pending desktop completion

- **WHEN** a desktop task is full but its completion callback is not persisted
- **THEN** the display holds the full bar without fabricating rewards, task
  counts, next activity, or reports

### Requirement: Partial desktop operation availability

The dashboard SHALL distinguish fully eligible, wholly ineligible, and partially
eligible desktop characters. When some but not all operations are eligible, it
SHALL show
which automatic level, automatic act, manual brag, motto, and guild operations
are available, with safe reasons for gated operations. Brag and profile editors
SHALL be gated by their requested operation rather than an all-operations
summary.

Local-only provenance SHALL continue to disable every online action and show
the fresh-official-import recovery instruction. Presentation SHALL NOT itself
send requests, change provenance, or confer eligibility. Browser presentation
and the concise uniform-state desktop summary SHALL remain unchanged.

#### Scenario: Showing manual-only availability

- **WHEN** a fresh desktop managed import has eligible manual brag but gated
  automatic, motto, and guild operations
- **THEN** the dashboard identifies partial availability, exposes manual brag,
  and explains the unavailable operations without labeling all actions blocked

#### Scenario: Opening a supported motto editor with guild gated

- **WHEN** motto is eligible and guild is ineligible for a Pemptus character
- **THEN** the existing motto editor remains usable while the guild action
  reports its own safe blocking reason

#### Scenario: Showing uniform eligibility

- **WHEN** every desktop online operation has the same eligibility outcome
- **THEN** the dashboard retains a concise overall summary without unnecessary
  mixed-operation detail

#### Scenario: Showing usable supported Pemptus operations

- **WHEN** an otherwise eligible unadapted, spelling-only, or placeholder-only
  Pemptus import has valid reporting/motto coverage and public-confirmation guild readiness
- **THEN** all five operations are available with existing controls and no
  obsolete import-path mismatch or automatic-fork warning

#### Scenario: Showing a local-only Pemptus timeline

- **WHEN** a Pemptus managed timeline has durable local-only provenance
- **THEN** every online action is disabled and the dashboard instructs the
  operator to use a fresh official-client import

#### Scenario: Refreshing partial availability

- **WHEN** the dashboard renders or refreshes eligibility details
- **THEN** it sends no leaderboard request and does not alter character state

### Requirement: Scrolling overflowing dashboard content

The dashboard SHALL allow users to read vertically overflowing content in
expanded full-layout panes and in the compact layout's combined Character
pane. In the full layout, Tab and Shift+Tab SHALL move keyboard scroll focus
forward and backward among expanded panes, wrapping at either end. The
focused pane SHALL be visibly distinguishable. Up and Down SHALL scroll the
focused pane by one content row; PageUp and PageDown SHALL scroll it by one
visible content page. Scrolling SHALL NOT alter character state or pane
collapse state.

When the user scrolls the mouse wheel over an expanded full-layout pane, the
dashboard SHALL scroll that pane. In the compact layout, keyboard and mouse
wheel scrolling SHALL apply to the combined Character pane.

Scroll offsets SHALL be independent for each pane and SHALL be kept within the
available content range as the terminal is resized or content changes.

#### Scenario: Navigating keyboard scroll focus

- **WHEN** the user presses Tab or Shift+Tab in the full dashboard
- **THEN** keyboard scroll focus moves to the next or previous expanded pane,
  wrapping at the first and last panes
- **AND** the focused pane is visibly distinguished

#### Scenario: Scrolling a focused full-layout pane

- **WHEN** an expanded full-layout pane has more rendered content rows than
  its visible area and the user presses Up, Down, PageUp, or PageDown
- **THEN** only the focused pane scrolls in the requested direction
- **AND** its content remains within the available scroll range

#### Scenario: Scrolling with the mouse wheel

- **WHEN** the user moves the mouse wheel over an expanded full-layout pane
- **THEN** only the pane under the pointer scrolls
- **AND** its scroll offset remains within the available content range

#### Scenario: Scrolling compact Character content

- **WHEN** the compact dashboard's Character content exceeds its visible area
- **THEN** keyboard scrolling and mouse-wheel scrolling allow the user to read
  all content in that pane

#### Scenario: Keeping scroll position valid

- **WHEN** terminal resizing or refreshed content reduces the available
  content range of a pane
- **THEN** that pane's scroll offset is clamped to a valid position
- **AND** offsets for other panes remain unchanged

#### Scenario: Keeping dashboard actions and state safe

- **WHEN** the user scrolls or changes keyboard scroll focus
- **THEN** no character state is changed and existing pane collapse shortcuts
  continue to work
