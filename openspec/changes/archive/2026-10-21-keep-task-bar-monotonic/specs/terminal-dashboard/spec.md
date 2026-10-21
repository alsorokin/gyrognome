# Spec Delta

## MODIFIED Requirements

### Requirement: Predicted current-task progress

The dashboard SHALL display a predicted position for the current task progress
bar between persisted state reads, derived from monotonic time elapsed since it
observed the most recent persisted state. Prediction SHALL use observed task
position, duration, compatibility profile, and rested timing information; it
SHALL NOT invoke simulation, acquire a character lock, or write character
state.

For browser characters, normal prediction SHALL advance at wall-clock rate.
For desktop characters, normal prediction SHALL advance at 100 task
milliseconds per 109.375 elapsed milliseconds. While rest is available, the
applicable rate SHALL be doubled. Prediction SHALL split elapsed time at
estimated rest exhaustion, using normal rate thereafter. It SHALL NOT apply a
full 2x rate beyond the observed bank's remaining duration.

The prediction SHALL be display-only. It SHALL NOT be persisted, included in
any leaderboard report, or used to determine task completion, the
recent-task-update indicator, lifecycle eligibility, or manual-brag
eligibility.

The prediction SHALL apply only to the task progress bar, in both the full
Progress pane and the compact Character pane's dedicated task-bar row. The
experience, encumbrance, plot, and quest bars, the completed-task count, the
activity description, the character stats, equipment, inventory, spells,
plots, and quests SHALL change only when a newer persisted state is read.

The predicted position SHALL NOT exceed the current task's duration. When the
predicted position reaches that duration, the dashboard SHALL display a full
task bar and SHALL hold it there until a newer persisted state is read.

The dashboard SHALL predict only while a local runtime owns the character. If
not owned, or ownership is unknown, it SHALL show persisted task position
unchanged.

When the dashboard reads a newer persisted state, it SHALL discard the previous
prediction and re-anchor to the newly observed state.

Within a single task, identified by an unchanged completed-task count and task
duration, the displayed task position SHALL NOT decrease, including across
re-anchors and regardless of rested state. When a re-anchor finds the
displayed position ahead of the newly anchored prediction, the dashboard SHALL
display the greater of the newly anchored prediction and the previously
displayed position advancing at half the applicable predicted rate, so the
displayed bar slows down until the anchored prediction catches up and then
follows it. When the task identity changes, the dashboard SHALL display the
newly persisted position directly, including a reset to the start of the next
task.

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

