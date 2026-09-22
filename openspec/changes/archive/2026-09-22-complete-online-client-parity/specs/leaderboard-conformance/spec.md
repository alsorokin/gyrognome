# Spec Delta

## MODIFIED Requirements

### Requirement: Controlled anti-cheat conformance evidence

The project SHALL define a documented, explicitly confirmed Playwright-harness
procedure for external leaderboard conformance experiments. The official
browser client SHALL create the newly created, disposable online character and
its passkey SHALL remain only in the ephemeral browser experiment. The
procedure SHALL compare browser and Gyrognome report histories across initial
load, pause, restart, delayed callbacks, task completion, level-up, act
completion, manual bragging, and motto change. It SHALL additionally exercise
an accepted non-empty guild-designation submission, an accepted empty
guild-designation submission, and a deliberately invalid designation
through the official browser, record their request shape and sanitized
accepted or rejected outcome categories, and restore the disposable
character to no guild before completion. It SHALL
record only credential-free observations and SHALL require evidence that the
Gyrognome character appears in the normal leaderboard population rather than
the cheater population. Native `newguy` character-generation support is not
required.

#### Scenario: Running an external conformance experiment

- **WHEN** an operator explicitly confirms a disposable-character experiment
- **THEN** the harness creates the character through the official browser,
  sends no reports or guild requests for any existing managed character, and
  records credential-free comparison, guild-outcome, and
  leaderboard-classification evidence for the disposable character

#### Scenario: Observing accepted and rejected guild submissions

- **WHEN** the disposable browser character submits a testable existing guild
  designation, submits a deliberately invalid designation, and then submits an
  empty designation
- **THEN** the evidence records sanitized request field names, operation order,
  safe accepted or rejected categories, and normalized response fingerprints
  that remove the current and submitted designations before hashing, without
  recording either designation when it would identify a private guild, the
  passkey, signed URL, or raw response body

#### Scenario: Guild response fingerprints are ambiguous

- **WHEN** normalized accepted and rejected guild responses do not produce
  distinct credential-safe fingerprints
- **THEN** guild conformance fails and production guild actions remain disabled

#### Scenario: Guild cleanup cannot be confirmed

- **WHEN** the harness cannot confirm that the disposable character has no
  guild after the empty designation is submitted
- **THEN** guild conformance fails and production guild actions remain disabled

#### Scenario: Evidence is incomplete or classified as cheating

- **WHEN** a required report or guild scenario lacks passing evidence or the
  disposable character is classified in the cheater population
- **THEN** the conformance gate fails and the corresponding general online
  operations remain unsupported
