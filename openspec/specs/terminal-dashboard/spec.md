# terminal-dashboard Specification

## Purpose

Provide an interactive, credential-safe terminal view of a locally managed
character and its runtime lifecycle without duplicating simulation behavior.

## Requirements

### Requirement: Case-insensitive dashboard letter shortcuts

The dashboard and its character selector SHALL recognize both ASCII cases of
each existing letter shortcut, including Ctrl+C, regardless of Caps Lock.
Case-insensitive matching SHALL preserve existing action eligibility,
confirmation requirements, modifier handling, and key-event-kind handling.
Non-letter controls SHALL retain their existing behavior.

#### Scenario: Dashboard actions with Caps Lock enabled

- **WHEN** the dashboard receives `Q`, `B`, `M`, `G`, `S`, or `A` outside a
  profile editor
- **THEN** it handles the input as the corresponding lowercase quit, brag,
  motto, guild, lifecycle, or autostart shortcut
- **AND** existing eligibility checks and confirmation requirements still apply

#### Scenario: Selecting a character with uppercase shortcuts

- **WHEN** the character selector receives `J`, `K`, or `Q`
- **THEN** it performs the same next, previous, or cancel action as `j`, `k`, or
  `q`, respectively

#### Scenario: Interrupting with an uppercase control character

- **WHEN** the dashboard, character selector, or profile editor receives Ctrl+C
  represented with uppercase `C`
- **THEN** it performs the same quit or cancel behavior as Ctrl+C represented
  with lowercase `c`
- **AND** an open profile editor does not submit its contents

#### Scenario: Keeping non-letter controls unchanged

- **WHEN** Caps Lock is enabled and the user invokes Enter, Escape, arrow keys,
  PageUp, PageDown, Tab, Shift+Tab, or F1 through F6 where supported
- **THEN** each control retains its existing action

### Requirement: Preserve profile input case

Motto and guild editors SHALL preserve the exact case of printable text input,
including uppercase letters that are shortcuts outside the editor.
Case-insensitive shortcut matching SHALL NOT convert editor text or execute
dashboard letter actions while editing.

#### Scenario: Typing uppercase shortcut letters in a profile editor

- **WHEN** a user types `QBM GSA` into an open motto or guild editor
- **THEN** the text is inserted with that exact case
- **AND** no quit, brag, profile-opening, lifecycle, or autostart action is
  triggered

#### Scenario: Submitting mixed-case profile text

- **WHEN** a user enters valid mixed-case text in a motto or guild editor and
  submits with Enter
- **THEN** the submitted text retains its case and passes through existing
  validation and action handling

### Requirement: Clean inventory label spacing

The dashboard SHALL omit trailing literal spaces from inventory names when rendering inventory labels. It SHALL preserve the original inventory names in stored state and compatibility data.

#### Scenario: Rendering an inventory name with trailing spaces

- **WHEN** an inventory name ends with one or more literal spaces
- **THEN** the dashboard displays the name without those trailing spaces before its quantity
- **AND** the inventory name in stored state remains unchanged

### Requirement: Dashboard opening and state overview

The system SHALL provide an interactive terminal dashboard for a specified
managed character. When an identifier is supplied, it
SHALL open that character directly. The dashboard SHALL display the persisted
credential-safe identity, current activity, progress bars, equipment,
inventory, spells, plot, and local runtime service status.
It SHALL report an actionable error when the requested character is not registered.

#### Scenario: Opening a registered character

- **WHEN** a user opens the dashboard for a registered managed character
- **THEN** the system renders its current credential-safe state and runtime
  service status in the terminal

#### Scenario: Opening an unknown character

- **WHEN** a user opens the dashboard with an identifier that is not registered
- **THEN** the system reports that the managed character was not found and does
  not enter the interactive terminal view

### Requirement: Dashboard character selection and recency

The `dashboard` command SHALL accept a character identifier or `--resume`/`-r`,
but not both. Without either, it SHALL show a credential-safe character list
with stable identifiers, last-access times (`Never` if unopened), and runtime
activity, ordered by access time descending then identifier ascending. Resume
SHALL open the first previously opened character in that same order, or show
the list if none has been opened.

#### Scenario: Selecting a character before opening the dashboard

- **WHEN** a user invokes `gyro dashboard` without an identifier and
  selects a registered character from the presented list
- **THEN** the system opens the dashboard for that selected character

#### Scenario: Viewing character recency and activity before selection

- **WHEN** a user invokes `gyro dashboard` without an identifier or resume
  flag and multiple managed characters are registered
- **THEN** the selection list displays each character's last accessed time and
  whether that character is currently active
- **AND** the characters are ordered from most recently opened in the
  dashboard to least recently opened, with never-opened characters last

#### Scenario: Cancelling dashboard selection

- **WHEN** a user invokes `gyro dashboard` without an identifier and cancels
  the character-selection flow, including a resume fallback
- **THEN** the system exits without opening a dashboard or changing any
  character data or last-access timestamps

#### Scenario: Opening the dashboard with no registrations

- **WHEN** a user invokes `gyro dashboard` without an identifier, with or
  without a resume flag, and no managed characters are registered
- **THEN** the system reports that no managed characters are available and
  does not enter an interactive dashboard

#### Scenario: Resuming with either flag spelling

- **WHEN** at least one registered character has been opened in the dashboard
  and the user invokes
  `gyro dashboard --resume` or `gyro dashboard -r`
- **THEN** the system opens the first character in the selection list's
  last-access ordering without displaying the selector
- **AND** the existing refresh interval option remains available

#### Scenario: Resuming without a previous opening

- **WHEN** registered characters exist but none has a recorded dashboard
  opening and the user requests resume
- **THEN** the system presents the normal selector instead of automatically
  opening its first entry

#### Scenario: Resuming after the most recently opened character was deleted

- **WHEN** the most recently opened character has been deleted, another
  previously opened character remains registered, and the user requests resume
- **THEN** the system opens the remaining character with the most recent
  dashboard-access time without displaying the selector

#### Scenario: No previously opened character remains

- **WHEN** all previously opened characters have been deleted but never-opened
  characters remain and the user requests resume
- **THEN** the system presents the normal selector

#### Scenario: Combining an identifier with resume

- **WHEN** the user supplies both a character identifier and either resume flag
- **THEN** the system reports a command-line argument conflict before entering
  terminal mode or changing last-access timestamps

### Requirement: Dashboard-only last-access recording

The system SHALL persist a successful dashboard opening's time in the local
store only after the initial view renders. Failed or cancelled openings and
later refreshes SHALL NOT update recency. Recording SHALL NOT change character
or worker state, start or stop a worker, acquire simulation ownership, or send
requests. New characters and legacy records without known openings SHALL be
unopened. Storage failures SHALL be reported, with terminal mode restored.

#### Scenario: Recording direct and selected openings across invocations

- **WHEN** a user successfully opens character A directly by ID or through the
  selector, exits, and invokes resume in a later process using the same data store
- **THEN** the system opens A directly

#### Scenario: Background progression does not change recency

- **WHEN** character A was opened more recently than B in the dashboard and B
  later receives worker updates or profile edits
- **THEN** both characters' last-access times remain unchanged, A stays ahead
  of B in the selector, and resume opens A

#### Scenario: A failed opening preserves recency

- **WHEN** character A was last opened and an attempt to open character B fails
  before its first dashboard view is rendered
- **THEN** neither character's last-access time changes and resume still opens A

#### Scenario: Recording an opening does not advance a character

- **WHEN** the system records a successfully opened character's last-access time
- **THEN** canonical state, profile values, rested accounting, worker timing
  baselines, and worker activity remain unchanged and no network request is issued

#### Scenario: Registering or upgrading characters without access history

- **WHEN** a character is newly registered or its legacy worker-derived
  timestamp is migrated without a known dashboard-opening time
- **THEN** its access time displays as `Never`, it sorts after opened
  characters, and resume does not treat it as previously opened

#### Scenario: Equal access times have a consistent order

- **WHEN** multiple characters have the same last-access time
- **THEN** the selector and resume resolve their order by stable identifier
  ascending

#### Scenario: Last-access storage fails

- **WHEN** reading or writing last-access metadata encounters a storage error
- **THEN** the system reports the error without silently falling back or
  claiming the opening was recorded
- **AND** any interactive terminal mode is restored before exit

### Requirement: Full-layout Journal and Adventure presentation

In the full layout, Journal SHALL show the plot caption and quest history;
Adventure SHALL omit duplicate plot and current-quest rows and separate
inventory from spells with a blank line. Expanded Journal SHALL show the bold
current quest first, then completed quests newest-first. Collapsed Journal
SHALL append a non-empty current quest to its plot title, refresh that title
with displayed state, and clip it within the header while retaining two rows
and the right-aligned F6 shortcut.

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

### Requirement: Recent task update indicator

The dashboard SHALL show stats in its identity header, right-aligned when they
fit and otherwise on a second row, separated by ` | `. After a persisted
completed-task update, it SHALL highlight currently displayed stats, equipment,
inventory, and spells that changed. The highlight SHALL persist across refreshes
until another completed task replaces it with the newly changed values.

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

### Requirement: Dashboard pane collapse controls

The full dashboard SHALL let the user independently collapse and expand the
Activity, Progress, Equipment, Details, Adventure, and Journal panes using F1
through F6, respectively. Each collapsible pane header SHALL display its
assigned hotkey right-aligned. The Status pane SHALL remain expanded and SHALL
NOT have a collapse hotkey. Collapsing a pane SHALL hide its content and
reduce its layout allocation while preserving the visibility and state of
every other pane.

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

#### Scenario: Discovering pane shortcuts

- **WHEN** the full dashboard layout is displayed
- **THEN** Activity, Progress, Equipment, Details, Adventure, and Journal show
  F1, F2, F3, F4, F5, and F6 respectively on the right side of their headers

### Requirement: Full-layout pane space allocation

In the full layout, Journal SHALL use at most one quarter of the usable right
column, with Adventure receiving the rest. Expanded Activity SHALL use four
rows, Progress seven, and Equipment at most eleven content rows. Expanded
Details SHALL fill remaining left-column space, reserving at least eight total
rows when possible; collapsed Details SHALL use only its two border rows.

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

### Requirement: Dashboard Keys and Status arrangement

The full layout SHALL place Keys left of Status in the bottom row, at one-third
and two-thirds widths respectively. Each pane SHALL use three rows when Keys
fits on one content row, or four when Keys wraps to two. Shortcut keys SHALL
be bold; confirmation warnings MAY increase the row height to keep safety
text visible.

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

### Requirement: Compact dashboard fallback

The dashboard SHALL retain its existing compact character and Status
view for terminals that do not use the full layout. The compact Keys pane
SHALL use one inner-content row when its complete shortcut line fits and SHALL
expand to two inner-content rows only when that line wraps.

#### Scenario: Viewing the compact dashboard

- **WHEN** the terminal is below the full-layout width threshold
- **THEN** the dashboard continues to render its compact character and Status
  view
- **AND** its Keys pane uses one content row when the shortcut line fits and
  two only when it wraps

### Requirement: Compact character content layout

The compact dashboard SHALL show only the labeled current quest and retain plot
information. Its percentage summary SHALL contain experience, encumbrance,
plot, and quest only, with a full-width, single-row task bar immediately above
it using the full-layout appearance and predicted percentage. Character content
SHALL separate the summary, equipment, inventory, spells, and plot with blank
lines; the bar and separators SHALL scroll with the pane. Full-layout
presentation SHALL remain unchanged.

#### Scenario: Showing only the current quest

- **WHEN** the compact dashboard displays a character with a current quest and
  several completed quests
- **THEN** it shows the current quest with the `Current quest:` label
- **AND** completed quest history is absent, including after scrolling
- **AND** plot information remains available

#### Scenario: Separating task progress

- **WHEN** the compact dashboard displays task progress at 75 percent
- **THEN** the percentage summary contains only experience, encumbrance, plot,
  and quest percentages
- **AND** the preceding row is a task progress bar labeled `Task 75%`, with the same
  fill treatment as full mode

#### Scenario: Separating compact sections

- **WHEN** compact Character content includes equipment, inventory, spells,
  and plot information
- **THEN** one empty row separates the percentage summary from equipment, equipment from
  inventory, and the spellbook from plot information

#### Scenario: Scrolling wrapped content and the task bar

- **WHEN** compact Character content wraps or exceeds the visible height
- **THEN** scrolling preserves the task bar's position relative to the
  percentage summary and equipment
- **AND** the bar and separator rows scroll with the rest of the content,
  without overwriting borders or other content

### Requirement: Visible rested progression

Both layouts SHALL display rested duration; Details and the selector SHALL use
`Rested: <time>`. Compact content SHALL show 2x only during active rest, and 1x
otherwise. Display SHALL remain read-only and within pane-height contracts:
estimates SHALL NOT consume or persist rest, advance state, reset timing, or
issue requests. Details omit ID/compatibility; the dashboard omits the
rested-timeline notice retained in inspection and documentation.

#### Scenario: Viewing a boosted character

- **WHEN** a running character has five hours of rest available
- **THEN** both layouts display its remaining rested duration, and compact
  Character content also displays the active 2x multiplier

#### Scenario: Viewing an inactive rested character

- **WHEN** a stopped character has earned rest
- **THEN** both layouts show the available bank without treating it as
  currently progressing, and compact Character content displays 1x

#### Scenario: Viewing an exhausted bank

- **WHEN** a refreshed or display-estimated bank reaches zero
- **THEN** the dashboard shows normal-speed status without fabricating task
  completion

#### Scenario: Selecting a rested character

- **WHEN** the character-selection list is opened
- **THEN** each entry shows only `Rested: <time>` using its read-only
  projected rested duration, including pending stopped-time accrual

#### Scenario: Viewing simplified character details

- **WHEN** a character's dashboard is shown
- **THEN** Details omits ID and compatibility-profile rows and both layouts
  omit the rested-timeline acceptance notice while retaining rested status
  and actionable online eligibility guidance

### Requirement: Predicted current-task progress

Between reads, the dashboard SHALL predict the owned character's task bar from
monotonic elapsed time and persisted task/profile/rest state, using profile
timing and splitting at rest exhaustion. Prediction SHALL be display-only: no
simulation, writes, task completion, or changes to other values/actions. It
SHALL re-anchor on reads, never rewind within a task, cap at task duration, and
redraw the current whole-percent position without replay.

#### Scenario: Advancing within a task

- **WHEN** a local runtime owns the character and monotonic time elapses
  between persisted state reads
- **THEN** the displayed task bar advances in whole-percent steps toward the
  current task's duration without the dashboard advancing the character itself

#### Scenario: Predicting a desktop task at the callback rate

- **WHEN** a local runtime owns a desktop character whose task has 10,000
  milliseconds remaining
- **THEN** the predicted task bar reaches full after about 10,938 milliseconds
  of elapsed time rather than 10,000

#### Scenario: Reaching the end of a task before new state is read

- **WHEN** the predicted position reaches the current task's duration and no
  newer persisted state has been read
- **THEN** the dashboard displays a full task bar and leaves the completed-task
  count, rewards, and activity description at their persisted values

#### Scenario: Re-anchoring on newer persisted state

- **WHEN** the dashboard reads a persisted state for a different task than the
  state the current prediction was anchored to
- **THEN** it discards the prediction and displays the newly persisted task
  position, activity, and rewards

#### Scenario: Never rewinding within a task

- **WHEN** an owned character's task bar displays 52 percent and the dashboard
  reads a persisted state for the same task at 50 percent
- **THEN** the displayed task bar does not drop below 52 percent

#### Scenario: Never rewinding while rested

- **WHEN** an owned character has active rest, and the dashboard reads a
  persisted state for the same task whose position is behind the displayed
  position and whose rested balance differs from the previous state
- **THEN** the displayed task bar does not decrease

#### Scenario: Slowing down while ahead of the worker

- **WHEN** an owned browser character without rest displays task position
  6,000 milliseconds and the dashboard reads the same task at 5,000
  milliseconds
- **THEN** one elapsed second displays 6,500 milliseconds rather than 6,000 or
  7,000
- **AND** after two elapsed seconds the anchored prediction has caught up and
  the bar advances at the normal rate thereafter

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

#### Scenario: Predicting boosted browser progress

- **WHEN** an owned browser character has at least one second of rest and its
  task has sufficient remaining duration
- **THEN** one elapsed second predicts 2,000 task milliseconds

#### Scenario: Predicting boosted desktop progress

- **WHEN** an owned desktop character has sufficient rest and 10,000 task
  milliseconds remaining
- **THEN** the predicted bar reaches full after about 5,469 elapsed
  milliseconds

#### Scenario: Predicting across bank exhaustion

- **WHEN** an owned browser character is observed with 250 milliseconds of
  rest and has sufficient task duration
- **THEN** one elapsed second predicts 1,250 task milliseconds rather than
  2,000

### Requirement: Live persisted-state refresh

The dashboard SHALL refresh character/profile state and runtime status on
bounded schedules. For an owned runtime, it SHALL read published live
character/rested state; otherwise, durable state. A saturated predicted task
bar SHALL trigger a bounded settling read no later than the regular refresh.
The dashboard SHALL not own, advance, or write simulation except for explicit
profile metadata actions, which leave simulation ownership unchanged.

#### Scenario: Viewing an active runtime

- **WHEN** the character's local runtime publishes new live state while the
  dashboard is open
- **THEN** the dashboard displays the newer state on a subsequent refresh
  without advancing the character itself, even if the runtime has not yet
  durably committed it

#### Scenario: Ignoring live state from a runtime that no longer owns the character

- **WHEN** live state exists but its publishing process no longer owns the
  character
- **THEN** the dashboard displays the last durably persisted state

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
  runtime an opportunity to publish the completed task, and then reads
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

The dashboard SHALL provide one confirmed contextual lifecycle action through
the existing user-service interface: start an inactive runtime, stop an active
one, or clear a failed state and start it. It SHALL expose no separate recovery
action and SHALL report success or actionable failure without exiting.

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

The dashboard SHALL not display credentials, raw saves/responses, signed URLs,
or unknown fields. Eligible characters SHALL support manual brag and motto/
guild editing through approved endpoints. Editors SHALL show current values,
accept printable non-control Unicode, submit on Enter, cancel on Escape, and
enforce encoding limits; empty guild input SHALL leave. Outcomes SHALL be
safely categorized. Refresh, navigation, rendering, and lifecycle actions
SHALL NOT issue requests.

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

Full-layout Details SHALL show character ID, elapsed task time, and Realm, but
not Quest target. Non-empty Motto and Guild values SHALL appear independently.
Elapsed time SHALL use compact days/hours/minutes/seconds, omit leading zero
units, and render zero as `0s`. Confirmed profile and lifecycle outcomes SHALL
remain visible across refreshes for at least five seconds.

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

The dashboard SHALL show the persisted continuation profile and safe online
eligibility, including gating/local-only reasons and fresh-import guidance. It
SHALL distinguish unavailable desktop history from since-import counters and
select profiles from persisted state. Prediction SHALL NOT dispatch completion,
create report eligibility, or replace a pending desktop full bar with an
assumed next task.

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

The dashboard SHALL distinguish full, partial, and no desktop eligibility.
Partial status SHALL identify availability and safe gating reasons for
automatic level/act, manual brag, motto, and guild; each explicit action SHALL
be gated by its own eligibility. Local-only provenance disables all online
actions and shows fresh-import guidance. Presentation SHALL not change
eligibility, provenance, or state; browser and uniform desktop summaries
remain unchanged.

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

Users SHALL be able to scroll overflowing expanded full-layout panes and the
compact Character pane by keyboard or mouse wheel. Tab/Shift+Tab SHALL cycle
visible focus among expanded panes and a no-focus position; initially none is
focused. Arrows scroll one row, PageUp/PageDown one page, and the pane under
the pointer scrolls on wheel. Per-pane offsets SHALL remain independently
clamped to content bounds. Scrolling SHALL not change character or collapse
state.

#### Scenario: Navigating keyboard scroll focus

- **WHEN** the user presses Tab or Shift+Tab in the full dashboard
- **THEN** keyboard scroll focus moves to the next or previous expanded pane,
  passing through a no-focus position when moving past either end
- **AND** any focused pane is visibly distinguished

#### Scenario: Leaving all panes unselected

- **WHEN** the full dashboard opens or keyboard focus cycles past either end
- **THEN** no pane is selected
- **AND** Up, Down, PageUp, and PageDown do not scroll any pane
- **AND** toggling pane collapse state leaves focus unselected
- **AND** the next Tab selects the first expanded pane, or Shift+Tab selects
  the last expanded pane

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

### Requirement: Adventure update auto-scroll

After a persisted task completion changes inventory or spells, expanded
full-layout Adventure SHALL reveal the first changed entry (inventory before
spells), using the smallest scroll needed for its wrapped rows. Manual Adventure
scrolling, including boundary attempts, pauses reveals for 30 seconds and
restarts the pause; other-pane activity does not. Paused/hidden updates SHALL
not replay. Auto-scroll SHALL not alter focus, other offsets, state, or network
activity.

#### Scenario: Revealing an off-screen new entry

- **WHEN** a persisted task completion adds an inventory item outside the
  expanded Adventure viewport and no manual-scroll pause is active
- **THEN** Adventure scrolls the item's rendered rows into view with the smallest
  necessary offset change
- **AND** keyboard focus and other pane offsets remain unchanged

#### Scenario: Revealing a changed spell

- **WHEN** a persisted task completion changes an off-screen spell rank without
  changing any inventory entry and Adventure auto-scroll is enabled
- **THEN** Adventure reveals that spell without changing the inline list layout

#### Scenario: Selecting among several updates

- **WHEN** one persisted task-completion refresh adds or changes several
  inventory items and spells
- **THEN** Adventure selects the first changed inventory entry in display order,
  or the first changed spell if no inventory entry changed

#### Scenario: Keeping a visible target in place

- **WHEN** a qualifying update's selected entry is fully visible
- **THEN** Adventure retains its current offset

#### Scenario: Revealing an entry taller than the viewport

- **WHEN** the selected updated entry wraps across more rows than Adventure can
  display at once
- **THEN** Adventure shows the beginning of that entry within its valid scroll
  range

#### Scenario: Respecting and extending manual scrolling

- **WHEN** the user manually scrolls expanded Adventure at time T and scrolls
  it again at T plus 20 seconds
- **THEN** qualifying updates observed before T plus 50 seconds do not cause
  automatic scrolling
- **AND** scroll attempts at a content boundary also restart the pause

#### Scenario: Resuming only for updates after expiration

- **WHEN** an update is observed during the manual-scroll pause and the pause
  subsequently expires without another qualifying update
- **THEN** Adventure does not move or replay the suppressed update
- **AND** a qualifying update observed at or after the expiration resumes
  automatic revealing

#### Scenario: Scrolling another pane

- **WHEN** the user scrolls Journal or another pane, or only changes scroll focus
- **THEN** Adventure's manual-scroll pause is not started or extended

#### Scenario: Refreshing without a new update

- **WHEN** the dashboard opens, refreshes without another completed-task update,
  or redraws an existing highlighted value
- **THEN** Adventure does not automatically change its offset

#### Scenario: Observing updates while Adventure is hidden

- **WHEN** a qualifying update is observed while Adventure is collapsed or the
  dashboard is compact
- **THEN** no automatic reveal occurs
- **AND** expanding Adventure or returning to full layout does not replay that
  update or change the compact scroll offset

### Requirement: Dashboard autostart preference

Both layouts SHALL show `Autostart: On/Off/Unavailable` apart from runtime and
document `a` toggle. Confirmation SHALL name target and state worker activity
is unchanged; Enter confirms, Escape cancels. The CLI interface SHALL
be used; refresh periodically and on external changes. Unavailable state SHALL
show diagnostics, not guesses. Outcomes SHALL remain visible for 5s. Errors
SHALL keep the dashboard open/state unchanged. Toggle SHALL not advance
simulation, edit profiles, or send requests.

#### Scenario: Discovering autostart in either layout

- **WHEN** a user views an enabled character in a full or compact dashboard
- **THEN** it shows `Autostart: On` independently of current service activity and documents the `a` shortcut

#### Scenario: Confirming an enable action

- **WHEN** a user presses `a` for an autostart-disabled inactive character and confirms with Enter
- **THEN** the dashboard enables autostart, refreshes its preference, reports the result, and leaves the worker inactive

#### Scenario: Cancelling a toggle

- **WHEN** a user presses Escape at an autostart confirmation
- **THEN** the dashboard leaves both autostart configuration and runtime activity unchanged

#### Scenario: Disabling startup without stopping

- **WHEN** a user confirms the autostart toggle for an enabled active character
- **THEN** the dashboard disables autostart without stopping the worker

#### Scenario: Configuration unavailable or action fails

- **WHEN** autostart state cannot be determined or a confirmed configuration action fails
- **THEN** the dashboard displays an actionable error without guessing the preference, changing displayed character state, or exiting

#### Scenario: Observing an external enablement change

- **WHEN** autostart is changed through the CLI or service-manager tools while the dashboard is open
- **THEN** the dashboard displays the new preference on a subsequent service-status refresh
