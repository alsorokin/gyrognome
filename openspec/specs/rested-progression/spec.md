# rested-progression Specification

## Purpose

Let managed characters recover a capped amount of missed progression through
future accelerated play while preserving durable state and credential-safe
inspection.

## Requirements

### Requirement: Automatic capped rest accumulation

Every managed browser and desktop character SHALL automatically accumulate
one second of rested time per second its worker is stopped or its computer is
asleep or hibernating, capped at 43,200 seconds, whether online or offline.
This SHALL apply without an enablement command.
Rest accumulation SHALL NOT advance game state or produce reports. Ordinary
scheduler stalls, task-completion callbacks, persistence, and report-delivery
delays while the computer remains awake SHALL NOT earn rest.

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

### Requirement: Rest accumulation baseline

New registrations SHALL begin with an empty bank and accrue rest from
registration onward. Existing records SHALL receive an empty bank and a timing
baseline when migrated; pre-migration history SHALL NOT count as rest. Profile
edits and read-only inspection SHALL NOT reset the rest baseline.

#### Scenario: Registering a new character

- **WHEN** a new character is registered
- **THEN** it starts with an empty bank and accrues rest from registration

#### Scenario: Initializing existing characters

- **WHEN** an existing record without rested metadata is migrated
- **THEN** its bank begins at zero at migration time without changing its
  canonical state, continuation, credentials, or eligibility

#### Scenario: Editing a stopped character's profile

- **WHEN** a stopped character's motto or guild changes during an eight-hour
  absence
- **THEN** the edit does not reduce the eight hours of earned rest

### Requirement: Active-time rested spending

Available rest SHALL double a running character's profile speed, accelerating
task completion effects, not only experience. Each awake second SHALL spend one
banked second, including processing and delays. Stopped or sleeping time SHALL
neither spend nor earn rest. When rest expires mid-interval, only covered time
SHALL receive the bonus; the remainder uses normal speed. Interrupted play
SHALL preserve unused rest before downtime accrues. Scheduling delays SHALL NOT
be repaid as catch-up.

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
timestamps. Successful commits SHALL atomically record rested accounting,
canonical state, random continuation, and applicable provenance/counters;
failed commits SHALL preserve the previous complete checkpoint. Graceful stop
SHALL record the latest balance and stop baseline for both profiles, even when
no task completes.

#### Scenario: Failing a boosted commit

- **WHEN** storing a boosted result fails
- **THEN** the prior game state and rested checkpoint remain intact and the
  runtime reports the failure

#### Scenario: Stopping without a completed task

- **WHEN** a browser or desktop worker stops gracefully partway through a task
- **THEN** its latest runtime state, remaining rest, and stopped timing
  baseline are persisted together

### Requirement: Restart recovery and timing safety

Restart SHALL recover the last complete checkpoint without restoring rest
spent on committed progression or duplicating accounted sleep. Abnormal exits
SHALL retain rollback guarantees; unobserved stopped time SHALL be estimated
from the last durable timing checkpoint, not as exact. Backwards clock movement
SHALL NOT create negative balances or repeat credits. Invalid timing or balance
data and clock-read failures SHALL produce explicit safe errors, never
fabricated rest or progression.

#### Scenario: Restarting after a committed boost

- **WHEN** a boosted advancement is committed and the worker restarts
- **THEN** the committed advancement and its spent rest remain paired, with
  only subsequent downtime added

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

Human-readable and JSON inspection SHALL expose available rest in milliseconds
and active multiplier (1 or 2), without credentials. A stopped character SHALL
report multiplier 1 while retaining its bank. Read-only inspection SHALL
include pending stopped-time accrual without persisting or consuming it.
Official save export/import SHALL NOT transfer rested balance or timing
metadata.

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
