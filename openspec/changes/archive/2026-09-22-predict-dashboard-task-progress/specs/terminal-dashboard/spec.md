# Spec Delta

## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Live persisted-state refresh

The dashboard SHALL refresh its displayed managed-character state and runtime
status at bounded periodic intervals and when the user requests a refresh. It
MAY read persisted character state and runtime service status on separate
bounded schedules. A user-requested refresh SHALL read both.

When a predicted task position reaches the current task's duration, the
dashboard SHALL wait a bounded settling interval before reading persisted
character state, so that the local runtime has an opportunity to commit the
completed task first. It SHALL separate successive reads triggered this way by
at least that settling interval, and SHALL NOT delay such a read beyond its
regular state-refresh interval.

It SHALL read the persisted canonical state and service status only; it SHALL
NOT own a character lock, advance simulation, or write character state.

#### Scenario: Viewing an active runtime

- **WHEN** the character's local runtime persists new state while the dashboard
  is open
- **THEN** the dashboard displays the newer persisted state on a subsequent
  refresh without advancing the character itself

#### Scenario: Refreshing an inactive runtime

- **WHEN** the character runtime is inactive and the user requests a refresh
- **THEN** the dashboard continues to display the last persisted state and
  updated inactive or failed service status

#### Scenario: Reading state and service status on separate schedules

- **WHEN** the dashboard reads persisted character state on a shorter schedule
  than runtime service status
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
