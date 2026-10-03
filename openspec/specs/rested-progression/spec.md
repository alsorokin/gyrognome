# rested-progression Specification

## Purpose

Let managed characters recover a capped amount of missed progression through
future accelerated play while preserving durable state and credential-safe
inspection.

## Requirements

### Requirement: Automatic capped rest accumulation

Every managed browser and desktop character SHALL automatically accumulate
one second of rested time per second its worker is stopped or its computer is
asleep or hibernating, capped at 43,200 seconds. This SHALL apply to offline
and online characters without an enablement command. Rest accumulation SHALL
NOT itself advance game state or produce reports. Ordinary scheduler stalls,
task-completion callbacks, persistence, and report-delivery delays while the
computer remains awake SHALL NOT earn rest.

New registrations SHALL begin with an empty bank and accrue rest from
registration onward. Existing records SHALL receive an empty bank and a timing
baseline when migrated; pre-migration history SHALL NOT be interpreted as
rest. Profile edits and read-only inspection SHALL NOT reset the rest baseline.

#### Scenario: Resting for eight hours

- **WHEN** a character with an empty bank has no running worker for eight
  hours
- **THEN** it has eight hours of rest available without changes to tasks,
  rewards, or random continuation

#### Scenario: Capping a long absence

- **WHEN** a character with two hours banked remains stopped for twenty hours
- **THEN** its available bank is twelve hours, not twenty-two

#### Scenario: Resuming after computer sleep

- **WHEN** a worker survives four hours of computer sleep or hibernation with
  three hours banked
- **THEN** seven hours are available after wake, no bank is spent during sleep,
  and the sleep is not applied as immediate game progress

#### Scenario: A delayed but awake worker

- **WHEN** the computer remains awake while scheduling, storage, or network
  delivery delays a worker
- **THEN** the delay does not increase its rested bank

#### Scenario: Initializing existing characters

- **WHEN** an existing record without rested metadata is migrated
- **THEN** its bank begins at zero at migration time without changing its
  canonical state, continuation, credentials, or eligibility

#### Scenario: Editing a stopped character's profile

- **WHEN** a stopped character's motto or guild changes during an eight-hour
  absence
- **THEN** the edit does not reduce the eight hours of earned rest

### Requirement: Active-time rested spending

A running character with available rest SHALL progress at twice its profile's
normal speed, accelerating tasks and their normal completion effects rather
than multiplying experience rewards alone. One real second of awake active
runtime SHALL spend one banked second, including time spent in ordinary runtime
processing and delays. Stopped and sleeping time SHALL NOT spend rest.

When a bank expires during an active interval, only the portion covered by
remaining rest SHALL receive the bonus; the remainder SHALL use normal speed.
Interrupted play SHALL preserve the unused balance before subsequent rest
accrues. Scheduling delays SHALL NOT be repaid as accelerated catch-up.

#### Scenario: Spending a full bank

- **WHEN** twelve hours of rested time are available and the worker runs awake
  for twelve hours without exceptional delays
- **THEN** the character receives twice its normal progression over that
  period and its bank reaches zero

#### Scenario: Stopping during boosted play

- **WHEN** a character starts with eight hours banked, runs for three hours,
  and stops for four hours
- **THEN** it has nine hours available on its next start

#### Scenario: Exhausting rest within an update

- **WHEN** an otherwise normally scheduled one-second browser update begins
  with 250 milliseconds of rest
- **THEN** it spends that rest and earns 1,250 virtual milliseconds,
  servicing twelve 100-millisecond ticks and retaining 50 milliseconds for
  the next tick rather than boosting the entire second

#### Scenario: Awake processing consumes time but earns no rest

- **WHEN** report delivery occupies ten awake seconds while the worker has
  twenty seconds of rest available
- **THEN** ten seconds are spent, none are earned, and normal profile delay
  limits prevent catch-up progression

### Requirement: Durable rested accounting

Rested balance, fractional virtual-time remainder, and timing metadata SHALL
be separate from canonical official-save data and general record-update
timestamps. Successful state commits SHALL atomically record the corresponding
rested accounting, canonical state, random continuation, and applicable
provenance/counters. A failed commit SHALL preserve the previous complete
checkpoint. Graceful stop SHALL record the latest balance and stop baseline
for both profiles, even when no task completes.

Restart SHALL recover the last complete accounting checkpoint without
restoring rest consumed by committed progression or duplicating an accounted
sleep interval. An abnormal exit SHALL retain the existing rollback
guarantees; its unobserved stop time SHALL be approximated from the last
durable runtime timing checkpoint rather than claimed as exact.

Backwards wall-clock movement SHALL NOT create negative balances or repeatedly
credit previously accounted time. Invalid persisted timing/balance data and
clock-read failures SHALL produce explicit safe errors rather than fabricate
rest or progression.

#### Scenario: Restarting after a committed boost

- **WHEN** a boosted advancement is committed and the worker restarts
- **THEN** the committed advancement and its spent rest remain paired, with
  only subsequent downtime added

#### Scenario: Failing a boosted commit

- **WHEN** storing a boosted result fails
- **THEN** the prior game state and rested checkpoint remain intact and the
  runtime reports the failure

#### Scenario: Stopping without a completed task

- **WHEN** a browser or desktop worker stops gracefully partway through a task
- **THEN** its latest runtime state, remaining rest, and stopped timing
  baseline are persisted together

#### Scenario: Recovering an abnormal exit

- **WHEN** a worker terminates without a stop checkpoint and is later
  restarted
- **THEN** it restores the last complete checkpoint and estimates rest since
  that timing checkpoint without replaying committed completions or reports

#### Scenario: Wall clock moves backwards

- **WHEN** the wall clock moves behind an already accounted stopped-time
  baseline
- **THEN** the bank is not credited for the backwards movement or again for
  traversing the previously accounted interval

### Requirement: Safe rested inspection

Managed-character inspection SHALL expose credential-free rested information
in human-readable and JSON output: remaining available rest in milliseconds
and the current active progression multiplier, either 1 or 2. A stopped
character SHALL have an active multiplier of 1 while retaining its available
bank. Read-only output SHALL account for pending stopped-time accrual without
persisting or consuming it. Official save export/import SHALL NOT transfer the
rested bank or timing metadata.

#### Scenario: Inspecting a boosted character

- **WHEN** a running character has five hours of rest remaining
- **THEN** managed inspection exposes 18,000,000 milliseconds available and
  active multiplier 2 without credentials

#### Scenario: Inspecting a stopped character

- **WHEN** a stopped character has earned eight hours since its baseline
- **THEN** repeated inspection exposes that available rest with active
  multiplier 1 without resetting its baseline or mutating game state

#### Scenario: Exporting and reimporting an official save

- **WHEN** a rested managed character is exported and its save is registered
  as a new character
- **THEN** the new registration has an empty rested bank and does not inherit
  runtime timing metadata
