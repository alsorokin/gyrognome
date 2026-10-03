# Spec Delta

## MODIFIED Requirements

### Requirement: Controlled offline advancement

The system SHALL advance a running managed character using elapsed time measured only by its active runtime. It SHALL supply explicit monotonic inputs to the selected profile, distinguishing real awake active time from simulated time under the rested-progression contract. Time while stopped or asleep SHALL NOT be applied as immediate catch-up; it SHALL accumulate capped rest for future active progression.

For browser characters, the system SHALL durably record each successful result with its rested accounting. Each update SHALL cap contributing real active time at the configured worker interval and discard excess scheduler delay permanently. Updates SHALL occur at the earlier of the interval, rested-bank exhaustion, and the tick-aligned task completion at the applicable speed. The runtime SHALL service stable 100-millisecond virtual ticks, persisting any fractional virtual-time remainder without padding it into extra progression. The interval SHALL remain the maximum single real-time contribution; virtual time earned SHALL be at most twice that contribution, with an earlier remainder available to complete a tick. Task-aligned scheduling SHALL preserve the browser's canonical/random continuation compared with equivalent fixed scheduling that supplies the same complete tick sequence. Arbitrary non-tick-aligned partitions SHALL NOT be claimed equivalent.

For desktop characters without rest, the runtime SHALL schedule callbacks at a fixed rate of one per 109.375 milliseconds, approximating the original client's 100 ms timer on default Windows timer resolution, so that time spent processing a callback does not lengthen the period. While rested, the runtime SHALL use a 2x virtual clock, with callbacks spaced by 54.6875 real milliseconds and callback elapsed inputs scaled by the applicable speed. Inputs spanning rest exhaustion SHALL account for their boosted and normal portions rather than applying 2x to the entire interval.

Each actual desktop callback SHALL credit its virtual elapsed input according to the unchanged 0..100-millisecond profile cap and SHALL preserve the full-bar-then-complete boundary. When callbacks fall behind schedule, the runtime SHALL skip missed callbacks rather than run them back to back. It SHALL NOT synthesize missed callbacks or accelerate callback frequency to drain a full task; rested pacing SHALL be the sole intentional acceleration. Restart and wake SHALL reestablish timing baselines while retaining the last durably recorded state and accounted rest. Time delayed by online delivery SHALL NOT become catch-up advancement or earned rest.

Desktop runtime state and rested accounting SHALL be durably recorded at least: when a callback dispatches completion; before any callback report is delivered; when local-only provenance is first recorded; on graceful stop; and at bounded checkpoints no farther than one awake second apart while callbacks are being serviced. Partial task progress SHALL NOT require a durable write on every callback. Between commit points, unrecorded progress and accounting SHALL roll back together on abnormal exit, without duplicating committed completion rewards or reports.

#### Scenario: Advancing while the runtime is active

- **WHEN** a character runtime remains active across advancement intervals
- **THEN** its state and rested accounting are durably recorded at the profile's commit points and are locally inspectable

#### Scenario: Restarting a stopped runtime

- **WHEN** a stopped runtime is started later
- **THEN** it resumes the last persisted profile-specific state with earned rest available, without instantly advancing through downtime

#### Scenario: Simulation cannot advance a state

- **WHEN** simulation reports an unsupported transition or other error
- **THEN** the runtime reports it, retains the last successful state/accounting, and does not fabricate advancement

#### Scenario: Delayed scheduled update

- **WHEN** scheduler delay or suspension postpones a callback
- **THEN** contributing awake time remains capped, excess scheduler delay is discarded, and independently measured sleep earns rest rather than immediate progression

#### Scenario: Unrepresentable elapsed duration

- **WHEN** an explicit worker duration cannot be represented in milliseconds
- **THEN** an elapsed-duration error leaves the last successful state and accounting intact

#### Scenario: Recording a task completion when it occurs

- **WHEN** a browser task completes before the real worker interval elapses at the applicable speed
- **THEN** the runtime advances to that completion and durably records the following task and rested accounting rather than waiting for the enclosing interval

#### Scenario: Advancing a task longer than the worker interval

- **WHEN** a browser task does not complete within the worker interval at the applicable speed
- **THEN** the runtime contributes at most one real interval and leaves the task active

#### Scenario: Preserving progression under boundary-aligned scheduling

- **WHEN** a browser character receives the same complete virtual-tick sequence through task-aligned and whole-interval scheduling
- **THEN** canonical state and random continuation match and any fractional remainder is retained without inventing progression

#### Scenario: Active task with no remaining time

- **WHEN** a browser task is already full or has less than one bounded tick left
- **THEN** the runtime services it at the next complete virtual tick without a zero-duration busy loop or excess credited time

#### Scenario: Pacing desktop callbacks independent of processing time

- **WHEN** an unboosted desktop runtime runs for one minute and each callback takes substantial but sub-period processing time
- **THEN** approximately 549 callbacks occur and approximately 54.9 seconds of task time can be credited before completion-only callback effects, rather than fewer callbacks spaced by sleep plus processing time

#### Scenario: Falling behind the desktop schedule

- **WHEN** a desktop callback finishes after later scheduled callback times have passed at either speed
- **THEN** the runtime uses the next future deadline and credits at most 100 virtual milliseconds without back-to-back catch-up callbacks

#### Scenario: Committing a desktop task completion

- **WHEN** a desktop callback dispatches task completion
- **THEN** state, random continuation, measured counters, and rested accounting are durably recorded before the next callback

#### Scenario: Committing before a desktop report

- **WHEN** a desktop callback produces a level or act report
- **THEN** its resulting state and rested accounting are committed before delivery, so restart cannot replay the callback or resend its report

#### Scenario: Not committing desktop partial progress per callback

- **WHEN** a desktop callback only advances a bar without reaching any event, provenance, or periodic commit point
- **THEN** it retains progress and accounting in memory rather than writing that callback separately

#### Scenario: Persisting a full desktop bar

- **WHEN** a desktop callback fills its task bar
- **THEN** a graceful stop or later commit retains the pending-completion state, and only the next actual callback dispatches completion

#### Scenario: Stopping a desktop runtime gracefully

- **WHEN** a desktop runtime receives a stop request
- **THEN** it durably records its latest state, including a pending full bar, remaining rest, and stopped timing baseline before exit

#### Scenario: Abnormal exit between desktop commits

- **WHEN** a desktop runtime exits after uncommitted progress
- **THEN** the next start restores the last complete state/accounting checkpoint without repeating recorded rewards or reports

#### Scenario: Pacing rested desktop callbacks

- **WHEN** a desktop worker has rest available for a minute with negligible processing delay
- **THEN** approximately twice as many callbacks occur as at normal speed, using the same profile transitions and per-callback cap

#### Scenario: Completing a rested desktop task

- **WHEN** a rested callback fills the desktop task bar
- **THEN** completion still waits for the next actual callback and does not also advance the following task

#### Scenario: Returning to normal desktop pacing

- **WHEN** the desktop worker exhausts its bank
- **THEN** subsequent pacing returns to 109.375 milliseconds without a missed-callback burst or resetting its task

#### Scenario: Checkpointing a long desktop task

- **WHEN** a desktop task remains incomplete across several awake seconds
- **THEN** periodic checkpoints persist matching partial progress and rest accounting without waiting for task completion
