## MODIFIED Requirements

### Requirement: Predicted current-task progress

The dashboard SHALL display a predicted position for the current task progress
bar between persisted state reads, derived from monotonic time elapsed since it
observed the most recent persisted state. The prediction SHALL be computed from
the observed persisted task position, task duration, and the character's
compatibility profile alone; it SHALL NOT invoke the simulation, acquire a
character lock, or write character state.

For browser characters, the predicted position SHALL advance at wall-clock
rate. For desktop characters, it SHALL advance at the desktop callback credit
rate of 100 milliseconds per 109.375 milliseconds of elapsed time.

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

