# local-character-runtime Specification

## Purpose

Provide durable, exclusively owned, local-only Progress Quest character
execution that safely connects a managed service lifecycle to deterministic
simulation.

## Requirements

### Requirement: Local character registration and persistence

The system SHALL import a valid supported browser or desktop save into the
invoking user's local managed-character store with a stable identifier. It
SHALL atomically persist canonical state, compatibility profile, random
continuation, safe online profile metadata, private credentials, import
provenance, reporting eligibility history, and runtime metadata. Browser
registration SHALL continue retaining the original JSON document. Desktop
registration SHALL retain validated data needed for continuation and
authentication without retaining a raw binary save or fabricating browser DNA.
Profile metadata SHALL remain independently updateable without overwriting
concurrent simulation progress.

Existing browser records SHALL migrate to the browser profile without changing
their state, random continuation, original JSON, credentials, or identifiers.
Unknown persisted profile/schema versions SHALL fail explicitly. Invalid
imports SHALL leave no partial registration; failed updates SHALL retain the
previous complete value. Desktop private data SHALL receive the same
user-private storage protections as browser credentials and SHALL be removed
with the character.

#### Scenario: Registering a valid browser save

- **WHEN** a user registers a valid browser `.pqw` save
- **THEN** one managed character with a stable identifier and browser profile
  is created and its safe canonical/profile state is inspectable

#### Scenario: Rejecting an invalid browser save

- **WHEN** a user registers malformed or invalid browser save data
- **THEN** an import error is reported without creating or modifying a
  character

#### Scenario: Recovering after an interrupted update

- **WHEN** persistence is interrupted while recording advancement
- **THEN** the character retains either its complete pre-advancement state or
  complete post-advancement state, including random continuation and eligibility

#### Scenario: Recovering after an interrupted profile update

- **WHEN** persistence is interrupted while changing motto or guild
- **THEN** the complete previous or requested profile remains, without
  corrupting simulation state

#### Scenario: Registering desktop private data atomically

- **WHEN** a supported desktop save includes online authentication
- **THEN** state, desktop profile, continuation, and private credentials are
  stored atomically and safe inspection omits credentials and raw properties

#### Scenario: Migrating an existing browser store

- **WHEN** a pre-profile database is opened by the new runtime
- **THEN** existing records receive the browser profile without changing their
  previous simulation, export, or online behavior

### Requirement: Safe managed-character removal

The local managed-character store SHALL remove every persisted record and
runtime metadata associated with a registered character when requested through
the character-administration interface. It SHALL reject removal of an unknown
character and SHALL preserve the character when removal cannot complete
atomically. It SHALL not remove a character while a local runtime owns it.

#### Scenario: Removing a registered inactive character

- **WHEN** character administration requests removal of a registered character
  that has no active runtime owner
- **THEN** the store removes the character's persisted state, original save
  document, and runtime metadata as one complete removal

#### Scenario: Failing a character removal

- **WHEN** character removal encounters a storage failure before completion
- **THEN** the system reports the failure and retains a complete readable
  managed character rather than a partial record

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
charged against task progress. It SHALL discard excess delay and preserve the
full-bar-then-complete callback boundary. When callbacks fall behind schedule,
the runtime SHALL skip the missed callbacks rather than run them back to back.
The runtime SHALL NOT synthesize missed callbacks or accelerate callback
frequency to drain a full task. Restart SHALL reestablish the timing baseline
while retaining the last durably recorded state. Time spent delayed by online
delivery SHALL NOT become catch-up advancement.

Desktop runtime state SHALL be durably recorded at least: when a callback
dispatches task completion; before any report produced by a callback is
delivered; when local-only provenance is first recorded; and on graceful
stop. The runtime is not required to record partial progress within a task.
Between those points, unrecorded partial-task progress MAY be lost on an
abnormal exit; such loss SHALL only roll the character back to an earlier
recorded state and SHALL NOT duplicate completion rewards or reports.

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

### Requirement: Exclusive character ownership

The system SHALL ensure that no more than one local runtime process owns a
managed character at a time. A competing start attempt SHALL identify the
existing ownership and leave the running character unaffected. Ownership from
an unexpectedly terminated process SHALL cease without requiring manual
database repair.

#### Scenario: Starting an already-owned character

- **WHEN** a user starts a runtime for a character already owned by another
  live local process
- **THEN** the system refuses the second start and reports that the character
  is already running

#### Scenario: Recovering ownership after an unexpected exit

- **WHEN** the owning runtime process exits unexpectedly
- **THEN** a later start can acquire ownership and resume from the last
  successfully persisted state

### Requirement: User-service lifecycle controls

The system SHALL provide command-line operations to start, stop, inspect
status for, and recover a managed character's local runtime through the
invoking user's service manager. Lifecycle operations SHALL report whether the
character is active and SHALL return actionable errors when the requested
character or user service cannot be managed.

#### Scenario: Starting a registered character

- **WHEN** a user starts a registered, unowned character
- **THEN** the system starts its user-scoped runtime service and reports the
  character as active

#### Scenario: Stopping an active character

- **WHEN** a user stops an active managed character
- **THEN** the system terminates its user-scoped runtime service after the last
  completed state update and reports the character as inactive

#### Scenario: Inspecting lifecycle status

- **WHEN** a user requests a managed character's runtime status
- **THEN** the system reports its identifier, persisted character identity, and
  whether its local runtime service currently owns it

### Requirement: Local-only runtime boundary

The runtime SHALL store and operate character data locally, except that an
eligible active online character SHALL attempt one request to its verified
official endpoint for each persisted profile-equivalent level-up or
act-completion event. Each report SHALL use its exact transition snapshot and
the current persisted motto. State SHALL be persisted before delivery, online
actions SHALL be serialized, failed delivery SHALL NOT be queued or retried,
and local advancement SHALL continue after failure. Offline and gated desktop
characters SHALL never report. Commands and diagnostics SHALL NOT expose
passkeys, account logins/passwords, authenticated URLs, or raw save fields.

#### Scenario: Reporting persisted online progression

- **WHEN** an eligible online runtime persists a level-up or act-completion
  event
- **THEN** it attempts one profile-compatible report with the current motto
  after persistence

#### Scenario: Changing a motto near an automatic report

- **WHEN** a motto action and automatic report contend for the online boundary
- **THEN** requests follow acquisition order and the later request uses the
  profile state established before its snapshot

#### Scenario: Running a managed online character

- **WHEN** a user starts an eligible character imported from an online save
- **THEN** it advances locally and sends only persisted level/act reports,
  without displaying credentials

#### Scenario: Running a managed offline character

- **WHEN** a user starts a character without online credentials
- **THEN** it advances locally with no network requests or credential exposure

#### Scenario: Failing automatic report delivery

- **WHEN** delivery fails or is rejected
- **THEN** persisted progress/profile state remains intact, no report is queued
  or retried, and later local advancement continues

#### Scenario: Invoking an explicit report submission

- **WHEN** an operator invokes a supported report or profile action for an
  eligible character
- **THEN** it can contact the verified official endpoint without stopping the
  runtime or exposing credentials

### Requirement: Durable desktop local-only advancement provenance

An online-originated desktop import SHALL be allowed to advance locally while
required classic progress-reporting evidence is unavailable. Its first
successful advancement under that gate SHALL atomically and durably mark it as
local-only. This restriction SHALL survive restart and later evidence updates
and SHALL disable every online operation for that managed import. Inspection,
registration, and startup without advancement SHALL NOT themselves mark a fork.

Later online use SHALL require a fresh supported official-client import under
matching passing evidence. The existing advanced record SHALL NOT be reset,
reenrolled, or made eligible by replacing credentials or clearing a flag.
Reports accumulated during gated advancement SHALL never be submitted.

#### Scenario: Advancing before classic evidence passes

- **WHEN** an online desktop character first advances while progress reporting
  is gated
- **THEN** its new state and local-only provenance are committed together and
  no request is sent

#### Scenario: Installing evidence after local advancement

- **WHEN** matching evidence becomes available for a previously forked import
- **THEN** it remains local-only and the operator is told to use a fresh
  official-client import rather than report its accumulated state

#### Scenario: Inspecting without advancing

- **WHEN** a desktop save is inspected or registered but never advanced
- **THEN** no local-only fork is recorded solely because evidence is absent
