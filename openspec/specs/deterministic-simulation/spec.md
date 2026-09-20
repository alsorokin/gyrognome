# deterministic-simulation Specification

## Purpose

Provide a browser-conformant, deterministic Progress Quest simulation core so
native characters can advance safely before runtime services or online reporting
are introduced.

## Requirements

### Requirement: Deterministic state advancement

The system SHALL advance a typed canonical character state by an explicitly
supplied duration without reading a wall clock, performing I/O, mutating a source
save document, or making a network request. It SHALL apply browser-compatible
task-bar advancement in bounded ticks and process task-completion effects in the
same order as the browser client. Equivalent nonzero partitions of a supplied
total duration SHALL produce the same canonical state and Alea continuation as
advancing by that total duration in one call. It SHALL expose ordered,
credential-free snapshots for browser leaderboard report transitions without
changing the resulting canonical state or compromising simulation purity.

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

#### Scenario: Capturing report-producing transitions

- **WHEN** a simulated advancement crosses a browser level-up or act-completion report point
- **THEN** the system exposes an ordered snapshot representing state at that
  exact browser transition while producing the same final canonical state as
  advancement without trace capture

### Requirement: Browser ruleset fidelity

The system SHALL resolve task selection, combat, rewards, equipment, spells,
quests, plots, and level-ups from a bundled Progress Quest ruleset with recorded
source revision and provenance. The ruleset SHALL be selected explicitly and
remain stable for a simulation run. Before state is persisted or used for a
leaderboard report, the canonical `bestspell` value SHALL equal the
browser-compatible name-and-rank display value of the learned spell with the
greatest product of its zero-based collection index plus one and its
Roman-numeral rank. Equal products SHALL retain the earliest spell. If no
spells are learned, `bestspell` SHALL be empty.

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
