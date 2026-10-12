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

### Requirement: Runtime active-time advancement

The system SHALL advance a running managed character using elapsed time
measured only by its active runtime. It SHALL supply explicit monotonic inputs
to the selected profile, distinguishing real awake active time from simulated
time under the rested-progression contract. Time while stopped or asleep SHALL
NOT be applied as immediate catch-up; it SHALL accumulate capped rest for
future active progression.

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

### Requirement: Browser runtime scheduling and persistence

For browser characters, the system SHALL publish each successful result with
its rested accounting as live runtime state, and SHALL durably record results
at the runtime commit boundaries. Each update SHALL cap contributing real
active time at the configured worker interval and discard excess scheduler
delay permanently. Updates SHALL occur at the earlier of the interval,
rested-bank exhaustion, and the tick-aligned task completion at the applicable
speed. The runtime SHALL service stable 100-millisecond virtual ticks,
retaining any fractional virtual-time remainder in its rested accounting
without padding it into extra progression.
The interval SHALL remain the maximum single real-time contribution; virtual
time earned SHALL be at most twice that contribution, with an earlier
remainder available to complete a tick. Task-aligned scheduling SHALL preserve
the browser's canonical/random continuation compared with equivalent fixed
scheduling that supplies the same complete tick sequence. Arbitrary
non-tick-aligned partitions SHALL NOT be claimed equivalent.

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

### Requirement: Desktop runtime callback pacing

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

### Requirement: Runtime durable commit boundaries

For both compatibility profiles, runtime state, random continuation,
measured counters, and rested accounting SHALL be durably recorded together at
least: before any report is delivered; when local-only provenance is first
recorded; when suspend or sleep is observed; on graceful stop; and at bounded
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

### Requirement: Managed character export

The system SHALL provide a command that exports a managed character by
identifier to a save file: `.pqw` for browser-profile characters and `.pq` for
desktop-profile characters. The output path SHALL default to `<save-name>`
with the matching extension in the current directory and MAY be overridden. If
the target exists, the command SHALL ask for confirmation and SHALL NOT
overwrite without it unless a force option is given. Files SHALL be written
atomically with user-private permissions, and a failed or declined export SHALL
leave any existing file unchanged. Exported files contain credentials, which
SHALL NOT be printed. Export SHALL NOT modify the stored character.

If the character's worker is running, export SHALL stop it, export a consistent
snapshot, and restart it, including when the export fails.

#### Scenario: Exporting a browser character

- **WHEN** a user exports a registered browser character
- **THEN** a `.pqw` is written that imports back to the same character state
  and retains its online credentials

#### Scenario: Declining an overwrite

- **WHEN** the target file exists and the user declines the prompt
- **THEN** the file is unchanged and the command exits without exporting

#### Scenario: Forcing an overwrite

- **WHEN** the target file exists and the force option is given
- **THEN** it is replaced atomically without prompting

#### Scenario: Exporting a running character

- **WHEN** the character's worker is running
- **THEN** it is stopped, the snapshot exported, and the worker restarted

### Requirement: Persistent per-character autostart

The system SHALL provide `gyro autostart <id> on|off` to enable or disable
persistent automatic startup of a registered character's user service.
Autostart SHALL remain independent of current service activity: changing it
SHALL NOT start, stop, or recover a worker, and Start/Stop/Recover SHALL NOT
change it. Repeating the same setting SHALL succeed without changing runtime
activity. Registration SHALL NOT automatically enable autostart.

Enabled characters SHALL start when the user's service manager starts,
including after reboot when that manager starts. The system SHALL NOT promise
startup before login unless account-wide lingering is configured. It SHALL
document that prerequisite without automatically enabling lingering or
requesting elevated privileges. Automatic startup SHALL retain existing
persisted-state, exclusive-ownership, reporting-eligibility, and rested-time
semantics, including no immediate downtime catch-up.

#### Scenario: Enabling an inactive character

- **WHEN** a user enables autostart for a registered inactive character
- **THEN** automatic startup is persistently enabled and the worker remains inactive

#### Scenario: Disabling an active character

- **WHEN** a user disables autostart for an active character
- **THEN** automatic startup is disabled and the worker remains active

#### Scenario: Returning after a reboot

- **WHEN** the user's service manager starts after reboot and a character has autostart enabled
- **THEN** it launches that character without opening the dashboard or manually starting it
- **AND** the worker resumes its persisted state without instant downtime advancement

#### Scenario: Stopping an enabled character

- **WHEN** a user stops a character with autostart enabled
- **THEN** it stops for the current session and retains automatic startup for the next user-manager startup

#### Scenario: Repeating a preference

- **WHEN** a user sets autostart to its already configured persistent value
- **THEN** the command succeeds without starting or stopping the worker

#### Scenario: Invalid character or unsuccessful configuration

- **WHEN** the identifier is invalid or unknown, the service template is unavailable, or automatic startup cannot be configured
- **THEN** the command reports an actionable error without claiming success or changing character data

### Requirement: Observable autostart configuration

`gyro status` SHALL report autostart separately from service activity and runtime
ownership in text and JSON. Its `autostart` JSON field SHALL distinguish
`enabled`, `disabled`, and `unavailable`. Enabled SHALL mean persistent startup,
not temporary runtime-only enablement. Unavailable or unsupported configuration
SHALL include an actionable diagnostic rather than being presented as disabled.
Status inspection SHALL NOT change enablement, character state, or account-wide
configuration.

#### Scenario: Enabled but stopped

- **WHEN** status is requested for an inactive character with persistent autostart enabled
- **THEN** output reports enabled autostart separately from inactive service activity

#### Scenario: External preference change

- **WHEN** persistent service enablement is changed outside Gyrognome
- **THEN** subsequent status reports reflect the service-manager configuration

#### Scenario: Unknown startup configuration

- **WHEN** startup configuration cannot be determined
- **THEN** status identifies autostart as unavailable with a diagnostic rather than asserting it is off
