## MODIFIED Requirements

### Requirement: Deterministic state advancement

The system SHALL advance a typed canonical character state by an explicitly
supplied duration without reading a wall clock, performing I/O, mutating a source
save document, or making a network request. It SHALL apply browser-compatible
task-bar advancement in bounded ticks and process task-completion effects in the
same order as the browser client. Equivalent nonzero partitions of a supplied
total duration SHALL produce the same canonical state and Alea continuation as
advancing by that total duration in one call.

#### Scenario: Advancing less than a task duration

- **WHEN** a caller advances a character by a duration that does not complete its active task
- **THEN** the system advances only task progress and leaves task count, elapsed task time, rewards, and random state unchanged

#### Scenario: Completing an active task

- **WHEN** a caller advances a character through completion of its active task
- **THEN** the system updates task count, elapsed time, progress effects, next activity, and random state in browser-compatible order

#### Scenario: Preserving offline purity

- **WHEN** a caller advances a character state
- **THEN** the system performs no wall-clock read, save-file write, database operation, or network request

#### Scenario: Partitioning elapsed duration

- **WHEN** a caller advances the same canonical state once by a total duration and separately by nonzero durations summing to that total
- **THEN** both advancement sequences produce the same canonical state and Alea continuation

### Requirement: Browser conformance checkpoints

The system SHALL verify deterministic simulation against committed synthetic
checkpoints derived from disposable browser sessions. Each checkpoint SHALL state
the input canonical state, supplied advancement sequence, selected ruleset
revision, expected canonical state, and expected Alea continuation state.
Checkpoint coverage SHALL include elapsed-duration partitioning and
task-completion boundaries. Fixtures SHALL NOT contain player saves, live
passkeys, browser profiles, or signed leaderboard requests.

#### Scenario: Replaying a checkpoint

- **WHEN** the system replays a committed checkpoint using its stated advancement sequence
- **THEN** its resulting canonical state and Alea continuation state match the checkpoint exactly

#### Scenario: Keeping reference data safe

- **WHEN** a browser observation is committed as a simulation checkpoint
- **THEN** it contains only synthetic or sanitized character data and no online bearer credential

#### Scenario: Verifying elapsed-time equivalence

- **WHEN** a conformance checkpoint defines equivalent total and partitioned advancement sequences
- **THEN** both sequences match the checkpoint's identical expected canonical state and Alea continuation state
