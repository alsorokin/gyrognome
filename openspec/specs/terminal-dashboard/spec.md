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
### Requirement: Collapsible dashboard panes

The full dashboard SHALL let the user independently collapse and expand the
Activity, Progress, Equipment, Details, Status, Adventure, and Journal panes
using F1 through F7, respectively. Each of those pane headers SHALL display
its assigned hotkey right-aligned. Collapsing a pane SHALL hide its content and
reduce its layout allocation while preserving the visibility and state of
every other pane. When both are expanded, Journal SHALL consume no more than
one quarter of the usable right-column height and Adventure SHALL receive the
remaining space. When expanded in the full dashboard layout, Activity SHALL
occupy exactly four terminal rows, consisting of two border rows and two
inner-content rows. Expanded Details SHALL fill all remaining left-column
height after allocating Activity, Progress, and the capped Equipment pane,
with at least six total rows when space permits. Collapsed Details SHALL
occupy only its two border rows. Progress SHALL occupy
exactly seven terminal rows, consisting of two border rows and five inner
progress-bar rows. Equipment SHALL receive flexible space when available but
its inner content height SHALL NOT exceed eleven rows, matching the maximum
number of equipment slots. The dashboard SHALL retain its existing compact
view for terminals that do not use the full layout.

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

#### Scenario: Allocating Activity height

- **WHEN** Activity is expanded in the full dashboard layout
- **THEN** it occupies exactly four terminal rows with two border rows and two
  inner-content rows

#### Scenario: Allocating Details height

- **WHEN** Details is expanded in the full dashboard layout
- **THEN** it fills all remaining left-column height after allocating the other
  panes, reserving at least six total rows when space permits
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
- **THEN** each collapsible pane header shows its corresponding F1 through F7
  hotkey on the right side

#### Scenario: Viewing the compact dashboard

- **WHEN** the terminal is below the full-layout width threshold
- **THEN** the dashboard continues to render its compact character view
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
profile metadata, and runtime status at bounded periodic intervals and when the
user requests a refresh. It MAY read persisted character/profile state and
runtime service status on separate bounded schedules. A user-requested refresh
SHALL read both.

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

- **WHEN** the character runtime is inactive and the user requests a refresh
- **THEN** the dashboard continues to display the last persisted state and
  updated inactive or failed service status

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
raw endpoint response bodies, or unrecognized raw save fields. It SHALL provide
an immediate manual-brag action and editable motto and guild actions for an
eligible managed character through the official leaderboard endpoint. Motto
and guild editors SHALL show the current persisted value, accept printable
non-control Unicode text, use Enter to submit, and use Escape to cancel.
Submitting an empty guild editor value SHALL request leaving the current guild;
there SHALL be no separate guild-leave action. The dashboard SHALL show only
credential-safe categorized outcomes. Dashboard refresh, navigation, rendering,
and lifecycle actions SHALL NOT independently make HTTP requests or report
leaderboard progress.

#### Scenario: Bragging from the dashboard

- **WHEN** an operator invokes the dashboard manual-brag action for an eligible
  inactive managed character
- **THEN** the dashboard immediately sends one browser-compatible manual-brag
  report and displays a credential-safe delivery outcome

#### Scenario: Changing a motto from the dashboard

- **WHEN** an operator opens the motto editor, changes or clears its value, and
  submits it for an eligible character
- **THEN** the dashboard performs one motto-change action, refreshes the
  displayed profile, and shows a credential-safe outcome

#### Scenario: Submitting a guild designation from the dashboard

- **WHEN** an operator submits a non-empty or empty guild designation for an
  eligible character
- **THEN** the dashboard performs one guild action, refreshes the displayed
  profile, and shows a credential-safe categorized outcome

#### Scenario: Cancelling a profile editor

- **WHEN** an operator presses Escape while editing a motto or guild
- **THEN** the dashboard closes the editor without changing state or sending a
  request

#### Scenario: Viewing an online-originated character

- **WHEN** a user opens the dashboard for a character imported from an online
  browser save
- **THEN** the dashboard shows only its credential-safe canonical state,
  profile metadata, and local service status without exposing the passkey or
  making a network request until the operator invokes an explicit online action

#### Scenario: Refreshing without bragging

- **WHEN** the dashboard refreshes, renders, navigates, or executes a lifecycle
  action without the operator invoking an explicit online action
- **THEN** it sends no leaderboard request
### Requirement: Concise profile and timing details

The full dashboard Details pane SHALL omit the Quest target line. It SHALL
display the character identifier and Last task elapsed. It SHALL display Motto
and Guild lines independently only when the corresponding persisted value is
non-empty.

Last task elapsed SHALL be formatted as a compact decomposition into days,
hours, minutes, and seconds rather than as one raw count of seconds. Leading
zero units SHALL be omitted and a zero duration SHALL render as `0s`.

#### Scenario: Viewing online profile details

- **WHEN** a managed character has both a motto and guild designation
- **THEN** Details displays Character ID, formatted Last task elapsed, Motto,
  and Guild without displaying Quest target

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
