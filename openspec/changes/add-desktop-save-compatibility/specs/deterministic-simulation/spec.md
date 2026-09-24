## MODIFIED Requirements

### Requirement: Deterministic state advancement

The system SHALL advance a typed canonical character state with explicit
inputs without reading a wall clock, performing I/O, mutating a source save,
or making a network request. Advancement SHALL select the character's
persisted compatibility profile and SHALL reject unsupported profiles or
mismatched random state.

For the browser profile, it SHALL apply browser-compatible task-bar
advancement by supplied duration in bounded ticks and process completion
effects in browser order. Equivalent nonzero partitions of a supplied total
duration SHALL produce the same canonical state and Alea continuation as
advancing by that total in one call. It SHALL expose ordered credential-free
browser report snapshots without changing final state or simulation purity.
Desktop advancement SHALL instead use the desktop callback contract; equal
elapsed totals with different callback sequences do not imply equivalence.

#### Scenario: Advancing less than a task duration

- **WHEN** a caller advances a browser character by a duration that does not
  complete its active task
- **THEN** only task progress changes; task count, elapsed task time, rewards,
  and random state remain unchanged

#### Scenario: Completing an active task

- **WHEN** a caller advances a browser character through task completion
- **THEN** task count, elapsed time, progress effects, next activity, and random
  state update in browser-compatible order

#### Scenario: Preserving offline purity

- **WHEN** a caller advances a character under either profile
- **THEN** no wall-clock read, save-file write, database operation, or network
  request occurs

#### Scenario: Partitioning elapsed duration

- **WHEN** a browser character advances once by a total duration and separately
  by nonzero durations summing to that total
- **THEN** both sequences produce identical canonical state and Alea
  continuation

#### Scenario: Capturing report-producing transitions

- **WHEN** browser advancement crosses a level-up or act-completion report point
- **THEN** its ordered snapshot represents that exact browser transition and
  final state matches advancement without trace capture

#### Scenario: Preventing cross-profile continuation

- **WHEN** a caller supplies a desktop state with browser random state or an
  unknown profile
- **THEN** advancement fails explicitly without substituting browser behavior

## ADDED Requirements

### Requirement: Deterministic desktop callback advancement

The desktop-6.4.4 profile SHALL consume an explicit ordered sequence of callback
elapsed inputs. When a task is full at callback entry, it SHALL process
completion effects without also advancing the next task during that callback.
Otherwise it SHALL advance the task by the nonnegative elapsed input capped
at 100 milliseconds, without processing completion until a later callback.
It SHALL discard excess elapsed time rather than carry it forward.

It SHALL expose ordered credential-free report snapshots at desktop report
points: level-up before subsequent completion effects, and act-completion
before the plot command is removed and the following loading task is selected.
Checkpoint restore SHALL preserve full-bar pending-completion state and
desktop randomness. Identical callback sequences SHALL remain equivalent
across call batching and checkpoint restoration.

#### Scenario: Filling a desktop task bar

- **WHEN** a 6,000-millisecond task starting at zero receives 60 callbacks of
  100 milliseconds
- **THEN** its bar is full but completion effects have not run
- **AND** callback 61 processes completion without advancing the next task

#### Scenario: Capping a delayed callback

- **WHEN** a desktop task not full at entry receives one callback delayed by
  10 seconds
- **THEN** at most 100 milliseconds is credited and the discarded time never
  advances later tasks

#### Scenario: Resuming pending completion

- **WHEN** a full desktop task bar is checkpointed and restored
- **THEN** the next callback completes that task exactly once, with the same
  rewards, report snapshots, and random continuation as uninterrupted execution

#### Scenario: Capturing desktop report order

- **WHEN** a callback includes a level-up followed by other completion effects
  or an act-completion queue transition
- **THEN** snapshots preserve the desktop transition state and event order,
  even when the final persisted state differs from an intermediate snapshot
