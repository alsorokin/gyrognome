## MODIFIED Requirements

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

