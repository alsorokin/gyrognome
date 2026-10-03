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
measured only by its active runtime. It SHALL supply explicit monotonic inputs
to the selected profile, distinguishing real awake active time from simulated
time under the rested-progression contract. Time while stopped or asleep SHALL
NOT be applied as immediate catch-up; it SHALL accumulate capped rest for
future active progression.

For browser characters, the system SHALL durably record each successful result
with its rested accounting. Each update SHALL cap contributing real active
time at the configured worker interval and discard excess scheduler delay
permanently. Updates SHALL occur at the earlier of the interval, rested-bank
exhaustion, and the tick-aligned task completion at the applicable speed. The
runtime SHALL service stable 100-millisecond virtual ticks, persisting any
fractional virtual-time remainder without padding it into extra progression.
The interval SHALL remain the maximum single real-time contribution; virtual
time earned SHALL be at most twice that contribution, with an earlier
remainder available to complete a tick. Task-aligned scheduling SHALL preserve
the browser's canonical/random continuation compared with equivalent fixed
scheduling that supplies the same complete tick sequence. Arbitrary
non-tick-aligned partitions SHALL NOT be claimed equivalent.

For desktop characters, the runtime SHALL schedule callbacks at a fixed rate
of one per 109.375 milliseconds, approximating the original client's 100 ms
timer on default Windows timer resolution, so that time spent processing a
callback does not lengthen the period. Each actual callback SHALL credit its
monotonic elapsed input, distinguishing real awake active time from simulated
time. Without rest, callbacks SHALL remain one per 109.375 milliseconds. While
rested, the runtime SHALL use a 2x virtual clock, with callbacks spaced by
54.6875 real milliseconds and callback elapsed inputs scaled by the applicable
speed. Inputs spanning rest exhaustion SHALL account for their boosted and
normal portions rather than applying 2x to the entire interval.

Each actual desktop callback SHALL credit its virtual elapsed input according
to the unchanged 0..100-millisecond profile cap and SHALL preserve the
full-bar-then-complete boundary. When callbacks fall behind schedule, the
runtime SHALL skip missed callbacks rather than run them back to back. It SHALL
NOT synthesize missed callbacks or accelerate callback frequency to drain a
full task; rested pacing SHALL be the sole intentional acceleration. Restart
and wake SHALL reestablish timing baselines while retaining the last durably
recorded state and accounted rest. Time delayed by online delivery SHALL NOT
become catch-up advancement or earned rest.

Desktop runtime state and rested accounting SHALL be durably recorded at
least: when a callback dispatches completion; before any callback report is
delivered; when local-only provenance is first recorded; on graceful stop; and
at bounded checkpoints no farther than one awake second apart while callbacks
are being serviced. Partial task progress SHALL NOT require a durable write on
every callback. Between commit points, unrecorded progress and accounting
SHALL roll back together on abnormal exit, without duplicating committed
completion rewards or reports.

#### Scenario: Advancing while the runtime is active

- **WHEN** a character runtime remains active across advancement intervals
- **THEN** its state and rested accounting are durably recorded at the
  profile's commit points and are locally inspectable

#### Scenario: Restarting a stopped runtime

- **WHEN** a stopped runtime is started later
- **THEN** it resumes the last persisted profile-specific state with earned
  rest available, without instantly advancing through downtime

#### Scenario: Simulation cannot advance a state

- **WHEN** simulation reports an unsupported transition or other error
- **THEN** the runtime reports it, retains the last successful
  state/accounting, and does not fabricate advancement

#### Scenario: Delayed scheduled update

- **WHEN** scheduler delay or suspension postpones a callback
- **THEN** contributing awake time remains capped, excess scheduler delay is
  discarded, and independently measured sleep earns rest rather than
  immediate game progress

#### Scenario: Unrepresentable elapsed duration

- **WHEN** an explicit worker duration cannot be represented in milliseconds
- **THEN** an elapsed-duration error leaves the last successful
  state/accounting intact

#### Scenario: Recording a task completion when it occurs

- **WHEN** a browser task completes before the worker interval elapses
- **THEN** the runtime advances to that completion and durably records the
  following task and rested accounting rather than waiting for the enclosing
  interval

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

#### Scenario: Pacing desktop callbacks independent of processing time

- **WHEN** a desktop runtime runs for one minute and each callback takes a
  substantial but sub-period time to process
- **THEN** approximately 549 callbacks occur and approximately 54.9 seconds of
  task time are credited, rather than fewer callbacks spaced by sleep plus
  processing time

#### Scenario: Falling behind the desktop schedule

- **WHEN** a desktop callback finishes after one or more later scheduled
  callback times have already passed
- **THEN** the runtime uses the next future deadline and credits at most
  100 virtual milliseconds without back-to-back catch-up callbacks

#### Scenario: Committing a desktop task completion

- **WHEN** a desktop callback dispatches task completion
- **THEN** state, random continuation, measured counters, and rested
  accounting are durably recorded before the next callback

#### Scenario: Committing before a desktop report

- **WHEN** a desktop callback produces a level or act report
- **THEN** its resulting state and rested accounting are committed before
  delivery, so restart cannot replay the callback or resend the report

#### Scenario: Not committing desktop partial progress per callback

- **WHEN** a desktop callback only advances the task bar without dispatching
  completion, producing a report, or recording local-only provenance
- **THEN** it retains progress and accounting in memory rather than writing
  that callback separately

#### Scenario: Persisting a full desktop bar

- **WHEN** a desktop callback fills its task bar
- **THEN** that pending-completion state is retained across a graceful stop
  or later commit point, and only the next actual callback dispatches
  completion

#### Scenario: Stopping a desktop runtime gracefully

- **WHEN** a desktop runtime receives a stop request
- **THEN** it durably records its latest state, including a pending full bar,
  remaining rest, and stopped timing baseline before exit

#### Scenario: Abnormal exit between desktop commits

- **WHEN** a desktop runtime exits abnormally after uncommitted partial
  progress
- **THEN** the next start restores the last complete state/accounting
  checkpoint without repeating recorded rewards or reports

#### Scenario: Pacing rested desktop callbacks

- **WHEN** a desktop worker has rest available for a minute with negligible
  processing delay
- **THEN** approximately twice as many callbacks occur as at normal speed,
  using the same profile transitions and per-callback cap

#### Scenario: Completing a rested desktop task

- **WHEN** a rested callback fills the desktop task bar
- **THEN** completion still waits for the next actual callback and does not
  also advance the following task

#### Scenario: Returning to normal desktop pacing

- **WHEN** the desktop worker exhausts its bank
- **THEN** subsequent pacing returns to 109.375 milliseconds without a
  missed-callback burst or resetting its task

#### Scenario: Checkpointing a long desktop task

- **WHEN** a desktop task remains incomplete across several awake seconds
- **THEN** periodic checkpoints persist matching partial progress and rest
  accounting without waiting for task completion

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
