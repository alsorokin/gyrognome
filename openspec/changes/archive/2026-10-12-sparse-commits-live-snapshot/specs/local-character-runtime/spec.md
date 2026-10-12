# Spec Delta

## MODIFIED Requirements

### Requirement: Browser runtime scheduling and persistence

For browser characters, the system SHALL publish each successful result with
its rested accounting as live runtime state, and SHALL durably record results
at the runtime commit boundaries. Each update SHALL cap contributing real active
time at the configured worker interval and discard excess scheduler delay
permanently. Updates SHALL occur at the earlier of the interval, rested-bank
exhaustion, and the tick-aligned task completion at the applicable speed. The
runtime SHALL service stable 100-millisecond virtual ticks, retaining any
fractional virtual-time remainder in its rested accounting without padding it
into extra progression. The interval SHALL remain the maximum single real-time
contribution; virtual time earned SHALL be at most twice that contribution,
with an earlier remainder available to complete a tick. Task-aligned
scheduling SHALL preserve the browser's canonical/random continuation compared
with equivalent fixed scheduling that supplies the same complete tick
sequence. Arbitrary non-tick-aligned partitions SHALL NOT be claimed
equivalent.

#### Scenario: Recording a task completion when it occurs

- **WHEN** a browser task completes before the worker interval elapses
- **THEN** the runtime advances to that completion and publishes the
  following task and rested accounting as live runtime state rather than
  waiting for the enclosing interval

#### Scenario: Advancing a task longer than the worker interval

- **WHEN** a browser task does not complete within the worker interval
- **THEN** the runtime contributes at most one real interval and leaves the
  task active

#### Scenario: Preserving progression under boundary-aligned scheduling

- **WHEN** a browser character receives the same complete virtual-tick
  sequence through task-aligned and whole-interval scheduling
- **THEN** canonical state and random continuation match and any fractional
  remainder is retained without inventing progression

#### Scenario: Active task with no remaining time

- **WHEN** a browser task is already full or has less than one bounded tick left
- **THEN** the runtime services it at the next complete virtual tick without
  a zero-duration busy loop or excess credited time

## ADDED Requirements

### Requirement: Runtime durable commit boundaries

For both compatibility profiles, runtime state, random continuation, measured
counters, and rested accounting SHALL be durably recorded together at least:
before any report is delivered; when local-only provenance is first recorded;
when suspend or sleep is observed; on graceful stop; and at bounded
checkpoints no farther than sixty awake seconds apart while the runtime is
advancing. Task completion and partial task progress SHALL NOT by themselves
require a durable write. Between commit points, unrecorded progress and
accounting SHALL roll back together on abnormal exit, without duplicating
committed completion rewards or reports.

#### Scenario: Not committing every task completion

- **WHEN** a task completes without producing a report or recording
  local-only provenance, and the last durable commit was less than sixty
  awake seconds ago
- **THEN** the runtime publishes its new live state without a durable
  database write

#### Scenario: Committing before a report

- **WHEN** advancement produces a level or act report
- **THEN** its resulting state and rested accounting are committed before
  delivery, so restart cannot replay the advancement or resend the report

#### Scenario: Checkpointing a long-running runtime

- **WHEN** a runtime advances for several minutes without reports
- **THEN** its state and rested accounting are durably recorded at least once
  per sixty awake seconds

#### Scenario: Persisting a full desktop bar

- **WHEN** a desktop callback fills its task bar
- **THEN** that pending-completion state is retained across a graceful stop
  or later commit point, and only the next actual callback dispatches
  completion

#### Scenario: Stopping a runtime gracefully

- **WHEN** a runtime receives a stop request
- **THEN** it durably records its latest state, including any pending full
  desktop bar, remaining rest, and stopped timing baseline before exit

#### Scenario: Abnormal exit between commits

- **WHEN** a runtime exits abnormally after uncommitted progress
- **THEN** the next start restores the last complete state/accounting
  checkpoint, losing at most about sixty awake seconds of progress, without
  repeating recorded rewards or reports

### Requirement: Live runtime state publication

A runtime that owns a character SHALL publish its current credential-safe
character state and rested accounting to a user-private location for local
observers. It SHALL publish at each task completion, after each durable
commit, and at intervals no longer than one awake second while advancing.
Publication SHALL replace the previous live state atomically, so observers
never read a partial value. It SHALL NOT write to the managed-character
database and SHALL NOT include online credentials or private save data. Live
state SHALL identify its owning runtime process. Observers SHALL treat it as
current only while that process still owns the character. A graceful stop
SHALL remove the live state after its final durable commit. A publication
failure SHALL NOT stop advancement or affect durable commits.

#### Scenario: Publishing a task completion

- **WHEN** an owned character completes a task
- **THEN** a local observer can read the following task, rewards, and rested
  accounting without a database write having occurred

#### Scenario: Ignoring stale live state after a crash

- **WHEN** a runtime exits abnormally and leaves live state behind
- **THEN** observers ignore it because its process no longer owns the
  character, and use the last durable state instead

#### Scenario: Keeping credentials out of live state

- **WHEN** an online character publishes live state
- **THEN** that state contains no passkey, authentication material, or raw
  save properties

## REMOVED Requirements

### Requirement: Desktop runtime commit boundaries

**Reason**: Replaced by the profile-independent "Runtime durable commit
boundaries" requirement, which no longer commits each desktop task completion
and widens the periodic checkpoint from one to sixty awake seconds.

**Migration**: Desktop runtimes follow "Runtime durable commit boundaries".
Live dashboard visibility of task completions comes from "Live runtime state
publication".
