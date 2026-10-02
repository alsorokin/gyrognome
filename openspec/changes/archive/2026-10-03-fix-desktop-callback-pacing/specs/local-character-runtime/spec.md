## MODIFIED Requirements

### Requirement: Controlled offline advancement

The system SHALL advance a running managed character using elapsed time
measured only by its active runtime. It SHALL supply explicit monotonic elapsed
inputs to the selected profile. Time while stopped SHALL NOT be applied as
catch-up.

For browser characters, the system SHALL durably record each successful
result. Each update SHALL cap elapsed time at the configured worker interval
and discard excess scheduler delay permanently. Updates SHALL occur at the
earlier of the interval and task completion, in whole bounded simulation ticks
and at least one tick. The interval SHALL remain the maximum single
advancement. Task-aligned scheduling SHALL preserve total simulated time and
the browser's canonical/random continuation compared with equivalent
whole-interval advancement.

For desktop characters, the runtime SHALL schedule callbacks at a fixed rate
of one per 109.375 milliseconds, approximating the original client's 100 ms
timer on default Windows timer resolution, so that time spent processing a
callback does not lengthen the period. Each actual callback SHALL credit the
monotonic time since the previous callback's simulation step finished, clamped
to 0..100 milliseconds, so that persistence and report delivery time is not
charged against task progress. It SHALL discard excess delay and preserve
the full-bar-then-complete
callback boundary. When callbacks fall behind schedule, the runtime SHALL skip
the missed callbacks rather than run them back to back. The runtime SHALL NOT
synthesize missed callbacks or accelerate callback frequency to drain a full
task. Restart SHALL reestablish the timing baseline while retaining the last
durably recorded state. Time spent delayed by online delivery SHALL NOT become
catch-up advancement.

Desktop runtime state SHALL be durably recorded at least: when a callback
dispatches task completion; before any report produced by a callback is
delivered; when local-only provenance is first recorded; and on graceful
stop. The runtime is not required to record partial progress within a task.
Between those points, unrecorded partial-task progress MAY be lost on an
abnormal exit; such loss SHALL only roll the character back to an earlier recorded
state and SHALL NOT duplicate completion rewards or reports.

#### Scenario: Advancing while the runtime is active

- **WHEN** a character runtime remains active across advancement intervals
- **THEN** its state is durably recorded at the profile's commit points and is
  locally inspectable

#### Scenario: Restarting a stopped runtime

- **WHEN** a stopped runtime is started later
- **THEN** it resumes the last persisted profile-specific state without downtime

#### Scenario: Simulation cannot advance a state

- **WHEN** simulation reports an unsupported transition or other error
- **THEN** the runtime reports it, retains the last successful state, and does
  not fabricate advancement

#### Scenario: Delayed scheduled update

- **WHEN** scheduler delay or suspension postpones a callback
- **THEN** elapsed contribution is capped to the browser worker interval or
  desktop 100-millisecond cap, with no later repayment

#### Scenario: Unrepresentable elapsed duration

- **WHEN** an explicit worker duration cannot be represented in milliseconds
- **THEN** an elapsed-duration error leaves the last successful state intact

#### Scenario: Recording a task completion when it occurs

- **WHEN** a browser task completes before the worker interval elapses
- **THEN** the runtime advances to that completion and durably records the
  following task rather than waiting for the enclosing interval

#### Scenario: Advancing a task longer than the worker interval

- **WHEN** a browser task does not complete within the worker interval
- **THEN** the runtime advances one interval and leaves the task active

#### Scenario: Preserving progression under boundary-aligned scheduling

- **WHEN** a browser character advances across task completions with
  task-aligned scheduling and separately for the same total in whole intervals
- **THEN** canonical state and random continuation match

#### Scenario: Active task with no remaining time

- **WHEN** a browser task is already full or has less than one bounded tick left
- **THEN** the runtime still supplies at least one whole tick rather than
  scheduling a zero-duration update

#### Scenario: Pacing desktop callbacks independent of processing time

- **WHEN** a desktop runtime runs for one minute and each callback takes a
  substantial but sub-period time to process
- **THEN** approximately 549 callbacks occur and approximately 54.9 seconds of
  task time are credited, rather than fewer callbacks spaced by sleep plus
  processing time

#### Scenario: Falling behind the desktop schedule

- **WHEN** a desktop callback finishes after one or more later scheduled
  callback times have already passed
- **THEN** the runtime runs the next callback at the next future scheduled
  time, crediting at most 100 milliseconds, without back-to-back catch-up
  callbacks

#### Scenario: Committing a desktop task completion

- **WHEN** a desktop callback dispatches task completion
- **THEN** the resulting state, random continuation, and measured counters are
  durably recorded before the next callback

#### Scenario: Committing before a desktop report

- **WHEN** a desktop callback produces a level or act report
- **THEN** the callback's resulting state is durably recorded before the
  report is delivered, so a restart cannot replay that callback or resend the
  report

#### Scenario: Not committing desktop partial progress per callback

- **WHEN** a desktop callback only advances the task bar without dispatching
  completion, producing a report, or recording local-only provenance
- **THEN** the runtime does not durably record that callback's result before
  the next commit point

#### Scenario: Persisting a full desktop bar

- **WHEN** a desktop callback fills its task bar
- **THEN** that pending-completion state is retained across a graceful stop or
  later commit point, and only the next actual callback dispatches completion

#### Scenario: Stopping a desktop runtime gracefully

- **WHEN** a desktop runtime receives a stop request
- **THEN** it durably records its latest in-memory state, including a full
  pending-completion bar, before exiting

#### Scenario: Abnormal exit between desktop commits

- **WHEN** a desktop runtime exits abnormally after uncommitted partial
  progress
- **THEN** the next start resumes the last durably recorded state, losing at
  most the uncommitted partial progress and never repeating a recorded
  completion or report
