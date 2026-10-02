# Spec Delta

## ADDED Requirements

### Requirement: Compact character content layout

The compact dashboard SHALL display only the canonical current quest with the
`Current quest:` label, rather than the full quest list or completed quest
history. It SHALL retain the plot information.

The compact percentage summary SHALL contain experience, encumbrance, plot,
and quest percentages only. Immediately above that summary, the dashboard
SHALL display task progress on a dedicated, single-row progress bar spanning
the Character pane's inner width. The bar SHALL use the same filled/unfilled
appearance and centered `Task <percent>%` label as the full-mode task bar,
and SHALL use the same display-only predicted task percentage.

The compact Character content SHALL include one empty line before equipment,
one empty line after equipment and before inventory, and one empty line after
the spellbook and before plot information. The task bar and empty separator
lines SHALL participate in the combined Character pane's normal scrolling.
These changes SHALL NOT change full-layout presentation.

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

## MODIFIED Requirements

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
Progress pane and the compact Character pane's dedicated task-bar row. The
experience, encumbrance, plot, and quest progress indicators, the completed-task
count, the activity description, the character stats, equipment, inventory,
spells, plots, and quests SHALL change only when a newer persisted state is read.

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
