# Spec Delta

## ADDED Requirements

### Requirement: Visible rested progression

Both full and compact dashboards SHALL display the available rested duration. Details and the character-selection list SHALL display only `Rested: <time>`, without an "available" suffix or multiplier. Compact Character content SHALL retain the active progression multiplier. Stopped characters SHALL show banked rest without claiming active boosted progression. Active characters with rest SHALL show 2x in compact Character content; exhausted banks SHALL show normal 1x there. Rested status SHALL remain available without changing existing dashboard pane height contracts. Details SHALL omit technical ID and compatibility-profile rows. Dashboard views SHALL omit the rested-timeline leaderboard acceptance notice; managed inspection and documentation SHALL retain that disclosure.

Rested presentation SHALL be read-only. Local countdown or pending-accrual estimates SHALL NOT consume or persist rest, advance game state, reset timing baselines, or issue requests.

#### Scenario: Viewing a boosted character

- **WHEN** a running character has five hours of rest available
- **THEN** both layouts display its remaining rested duration, and compact Character content also displays the active 2x multiplier

#### Scenario: Viewing an inactive rested character

- **WHEN** a stopped character has earned rest
- **THEN** both layouts show the available bank without treating it as currently progressing, and compact Character content displays 1x

#### Scenario: Viewing an exhausted bank

- **WHEN** a refreshed or display-estimated bank reaches zero
- **THEN** the dashboard shows normal-speed status without fabricating task completion

#### Scenario: Selecting a rested character

- **WHEN** the character-selection list is opened
- **THEN** each entry shows only `Rested: <time>` using its read-only projected rested duration, including pending stopped-time accrual

#### Scenario: Viewing simplified character details

- **WHEN** a character's dashboard is shown
- **THEN** Details omits ID and compatibility-profile rows and both layouts omit the rested-timeline acceptance notice while retaining rested status and actionable online eligibility guidance

## MODIFIED Requirements

### Requirement: Predicted current-task progress

The dashboard SHALL display a predicted position for the current task progress bar between persisted state reads, derived from monotonic time elapsed since it observed the most recent persisted state. Prediction SHALL use observed task position, duration, compatibility profile, and rested timing information; it SHALL NOT invoke simulation, acquire a character lock, or write character state.

For browser characters, normal prediction SHALL advance at wall-clock rate. For desktop characters, normal prediction SHALL advance at 100 task milliseconds per 109.375 elapsed milliseconds. While rest is available, the applicable rate SHALL be doubled. Prediction SHALL split elapsed time at estimated rest exhaustion, using normal rate thereafter. It SHALL NOT apply a full 2x rate beyond the observed bank's remaining duration.

Prediction SHALL remain display-only. It SHALL NOT be persisted, included in leaderboard reports, or determine completion, the recent-task-update indicator, lifecycle eligibility, or manual-brag eligibility.

Prediction SHALL apply only to the task bar, in both the full Progress pane and compact Character pane. Experience, encumbrance, plot and quest bars, completed-task count, activity, stats, equipment, inventory, spells, plots and quests SHALL change only when newer persisted state is read. The separately displayed rested estimate SHALL NOT change those values.

Predicted position SHALL NOT exceed task duration. At that duration, the dashboard SHALL hold a full bar until newer persisted state is read.

The dashboard SHALL predict only while a local runtime owns the character. If not owned, or ownership is unknown, it SHALL show persisted task position unchanged.

When newer persisted state or rested timing is read, the dashboard SHALL discard the previous prediction and re-anchor to the observation.

The dashboard SHALL schedule redraws in whole-percent steps for tasks of any duration, taking rest exhaustion into account. Delayed redraws SHALL show the current estimate rather than replay missed percentages.

#### Scenario: Advancing within a task

- **WHEN** a local runtime owns the character and time elapses between state reads
- **THEN** its displayed task bar advances in whole-percent steps at the applicable rate without advancing the character itself

#### Scenario: Predicting a desktop task at the callback rate

- **WHEN** an unboosted desktop character is owned and has 10,000 task milliseconds remaining
- **THEN** its predicted bar reaches full after about 10,938 elapsed milliseconds rather than 10,000

#### Scenario: Reaching the end of a task before new state is read

- **WHEN** prediction reaches task duration before newer state is read
- **THEN** the bar stays full while task count, rewards, and activity retain persisted values

#### Scenario: Re-anchoring on newer persisted state

- **WHEN** a state read changes task state or rested timing
- **THEN** the dashboard discards its old prediction and re-anchors to the newer observation

#### Scenario: Observing a character with no active runtime

- **WHEN** there is no local runtime owner or ownership cannot be determined
- **THEN** task position is displayed without prediction, even when rest is available

#### Scenario: Predicting without affecting other displayed values

- **WHEN** predicted task position and the rested countdown change
- **THEN** the recent-task indicator and all other game values continue to reflect persisted state only

#### Scenario: Recovering from a delayed redraw

- **WHEN** a redraw is delayed past its scheduled whole-percent boundary
- **THEN** the currently predicted percentage is shown without replaying intermediate percentages

#### Scenario: Predicting boosted browser progress

- **WHEN** an owned browser character has at least one second of rest and its task has sufficient remaining duration
- **THEN** one elapsed second predicts 2,000 task milliseconds

#### Scenario: Predicting boosted desktop progress

- **WHEN** an owned desktop character has sufficient rest and 10,000 task milliseconds remaining
- **THEN** the predicted bar reaches full after about 5,469 elapsed milliseconds

#### Scenario: Predicting across bank exhaustion

- **WHEN** an owned browser character is observed with 250 milliseconds of rest and has sufficient task duration
- **THEN** one elapsed second predicts 1,250 task milliseconds rather than 2,000
