# Spec Delta

## ADDED Requirements

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

## MODIFIED Requirements

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
