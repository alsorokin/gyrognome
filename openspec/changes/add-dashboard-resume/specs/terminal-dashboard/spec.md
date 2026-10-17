# Spec Delta

## MODIFIED Requirements

### Requirement: Dashboard character selection and recency

The `dashboard` command SHALL accept an optional managed character identifier
and equivalent `--resume` and `-r` flags. A resume flag and an explicit
identifier SHALL be mutually exclusive. When neither an identifier nor a
resume flag is supplied, it SHALL present a credential-safe interactive list
of registered characters and open the dashboard for the user-selected
character. When a resume flag is supplied, it SHALL open the most recently
opened registered character directly using the same last-access ordering as
the selection list. If no registered character has previously been opened
in the dashboard, it SHALL present the normal selection list.

The selection list SHALL display each character's credential-safe identity,
stable identifier, last accessed time, and whether its local runtime is
currently active. Last accessed SHALL mean the last successful dashboard
opening, not a worker update or profile edit. The selection list SHALL order
opened characters by last accessed time descending, followed by never-opened
characters, whose access time SHALL display as `Never`. Equal access times,
including never-opened entries, SHALL use stable identifier ascending as a
deterministic tie-breaker. Resume SHALL use this same order rather than a
separately remembered character identifier.

#### Scenario: Selecting a character before opening the dashboard

- **WHEN** a user invokes `gyro dashboard` without an identifier or resume
  flag and selects a registered character from the presented list
- **THEN** the system opens the dashboard for that selected character

#### Scenario: Viewing character recency and activity before selection

- **WHEN** a user invokes `gyro dashboard` without an identifier or resume
  flag and multiple managed characters are registered
- **THEN** the selection list displays each character's last accessed time and
  whether that character is currently active
- **AND** the characters are ordered from most recently opened in the
  dashboard to least recently opened, with never-opened characters last

#### Scenario: Cancelling dashboard selection

- **WHEN** a user invokes `gyro dashboard` without an identifier and cancels
  the character-selection flow, including a resume fallback
- **THEN** the system exits without opening a dashboard or changing any
  character data or last-access timestamps

#### Scenario: Opening the dashboard with no registrations

- **WHEN** a user invokes `gyro dashboard` without an identifier, with or
  without a resume flag, and no managed characters are registered
- **THEN** the system reports that no managed characters are available and
  does not enter an interactive dashboard

#### Scenario: Resuming with either flag spelling

- **WHEN** at least one registered character has been opened in the dashboard
  and the user invokes
  `gyro dashboard --resume` or `gyro dashboard -r`
- **THEN** the system opens the first character in the selection list's
  last-access ordering without displaying the selector
- **AND** the existing refresh interval option remains available

#### Scenario: Resuming without a previous opening

- **WHEN** registered characters exist but none has a recorded dashboard
  opening and the user requests resume
- **THEN** the system presents the normal selector instead of automatically
  opening its first entry

#### Scenario: Resuming after the most recently opened character was deleted

- **WHEN** the most recently opened character has been deleted, another
  previously opened character remains registered, and the user requests resume
- **THEN** the system opens the remaining character with the most recent
  dashboard-access time without displaying the selector

#### Scenario: No previously opened character remains

- **WHEN** all previously opened characters have been deleted but never-opened
  characters remain and the user requests resume
- **THEN** the system presents the normal selector

#### Scenario: Combining an identifier with resume

- **WHEN** the user supplies both a character identifier and either resume flag
- **THEN** the system reports a command-line argument conflict before entering
  terminal mode or changing last-access timestamps

## ADDED Requirements

### Requirement: Dashboard-only last-access recording

The system SHALL persist each character's last successful dashboard-opening
time across command invocations in the same local data store. Direct-ID,
selector-based, and resumed openings SHALL update that character's last
accessed time once the initial dashboard view has been rendered successfully.
Failed validation, failed terminal setup or initial rendering, and cancelled
selection SHALL NOT change any last-access timestamp. Subsequent dashboard
refreshes SHALL NOT update last accessed.

Recording a dashboard opening SHALL change only last-access metadata, not
canonical simulation state, profile values, rested accounting, or worker
timing baselines. It SHALL NOT start or stop a worker, acquire simulation
ownership, or send network requests. Worker updates, profile edits,
registration, and other commands SHALL NOT record dashboard accesses.
Newly registered characters SHALL begin as never opened.

Legacy saved-update timestamps SHALL NOT be interpreted as dashboard openings:
on upgrade, characters without known dashboard-opening times SHALL begin as
never opened. Failures reading or writing last-access metadata SHALL be
reported explicitly rather than silently treated as absent access history;
terminal mode SHALL be restored before an error exits the dashboard.

#### Scenario: Recording direct and selected openings across invocations

- **WHEN** a user successfully opens character A directly by ID or through the
  selector, exits, and invokes resume in a later process using the same data store
- **THEN** the system opens A directly

#### Scenario: Background progression does not change recency

- **WHEN** character A was opened more recently than B in the dashboard and B
  later receives worker updates or profile edits
- **THEN** both characters' last-access times remain unchanged, A stays ahead
  of B in the selector, and resume opens A

#### Scenario: A failed opening preserves recency

- **WHEN** character A was last opened and an attempt to open character B fails
  before its first dashboard view is rendered
- **THEN** neither character's last-access time changes and resume still opens A

#### Scenario: Recording an opening does not advance a character

- **WHEN** the system records a successfully opened character's last-access time
- **THEN** canonical state, profile values, rested accounting, worker timing
  baselines, and worker activity remain unchanged and no network request is issued

#### Scenario: Registering or upgrading characters without access history

- **WHEN** a character is newly registered or its legacy worker-derived
  timestamp is migrated without a known dashboard-opening time
- **THEN** its access time displays as `Never`, it sorts after opened
  characters, and resume does not treat it as previously opened

#### Scenario: Equal access times have a consistent order

- **WHEN** multiple characters have the same last-access time
- **THEN** the selector and resume resolve their order by stable identifier
  ascending

#### Scenario: Last-access storage fails

- **WHEN** reading or writing last-access metadata encounters a storage error
- **THEN** the system reports the error without silently falling back or
  claiming the opening was recorded
- **AND** any interactive terminal mode is restored before exit
