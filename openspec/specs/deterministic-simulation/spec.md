# deterministic-simulation Specification

## Purpose

Provide a browser-conformant, deterministic Progress Quest simulation core so
native characters can advance safely before runtime services or online reporting
are introduced.

## Requirements

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

### Requirement: Browser ruleset fidelity

The system SHALL resolve task selection, combat, rewards, equipment, spells,
quests, plots, and level-ups from a bundled Progress Quest ruleset with recorded
source revision and provenance. The ruleset SHALL be selected explicitly and
remain stable for a simulation run. Before state is persisted or used for a
leaderboard report, the canonical `bestspell` value SHALL equal the
browser-compatible name-and-rank display value of the learned spell with the
greatest product of its zero-based collection index plus one and its
Roman-numeral rank. Equal products SHALL retain the earliest spell. If no
spells are learned, `bestspell` SHALL be empty. Before state is persisted or
used for a leaderboard report, the canonical `beststat` value SHALL equal the
browser-compatible label-and-integer display value of the highest current prime
stat among `STR`, `CON`, `DEX`, `INT`, `WIS`, and `CHA`, and `Stats.best` SHALL
identify that same stat. Equal current values SHALL retain the earliest stat in
that browser order.

#### Scenario: Resolving a simulation outcome

- **WHEN** a simulation step requires a browser rule-table value
- **THEN** the system uses the selected bundled ruleset rather than an ad hoc or externally fetched value

#### Scenario: Reproducing a level transition

- **WHEN** task completion reaches an experience-bar level boundary
- **THEN** the system applies browser-compatible level, attribute, spell, progress-bar, and random-state changes using the selected ruleset

#### Scenario: Recording the strongest spell as Specialty

- **WHEN** a simulated state is persisted or produces a leaderboard report
- **THEN** the canonical `bestspell` value reflects the official ranked
  selection across all learned spells

#### Scenario: Resolving an equal-ranked Specialty

- **WHEN** two learned spells have equal ranked-selection products
- **THEN** the canonical `bestspell` value reflects the earlier spell in the
  learned-spell collection

#### Scenario: Reporting without learned spells

- **WHEN** a simulated state with no learned spells is persisted or produces a
  leaderboard report
- **THEN** the canonical `bestspell` value is empty

#### Scenario: Recording the current Prime Stat

- **WHEN** a simulated state is persisted or produces a leaderboard report after its prime statistics have changed
- **THEN** `beststat`, `Stats.best`, and the leaderboard Prime Stat value identify the highest current prime stat and its current integer value

#### Scenario: Resolving an equal current Prime Stat

- **WHEN** multiple current prime stats share the highest integer value at a persistence or report boundary
- **THEN** the canonical Prime Stat reflects the earliest tied name in `STR`, `CON`, `DEX`, `INT`, `WIS`, `CHA` order

### Requirement: Browser conformance checkpoints

The system SHALL verify deterministic simulation against committed synthetic
checkpoints derived from disposable browser sessions. Each checkpoint SHALL state
the input canonical state, supplied advancement sequence, selected ruleset
revision, expected canonical state, and expected Alea continuation state. Fixtures
SHALL cover elapsed-duration partitioning and task-completion boundaries.
Fixtures SHALL NOT contain player saves, live passkeys, browser profiles, or
signed leaderboard requests.

#### Scenario: Replaying a checkpoint

- **WHEN** the system replays a committed checkpoint using its stated advancement sequence
- **THEN** its resulting canonical state and Alea continuation state match the checkpoint exactly

#### Scenario: Keeping reference data safe

- **WHEN** a browser observation is committed as a simulation checkpoint
- **THEN** it contains only synthetic or sanitized character data and no online bearer credential

#### Scenario: Verifying elapsed-time equivalence

- **WHEN** a conformance checkpoint defines equivalent total and partitioned advancement sequences
- **THEN** both sequences match the checkpoint's identical expected canonical state and Alea continuation state

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
