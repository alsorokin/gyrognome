## Purpose

Create and locally register offline-only Progress Quest characters through a
browser-like terminal creation experience or explicit command-line inputs.

## ADDED Requirements

### Requirement: Offline character generation

The system SHALL generate a valid level-one canonical character from a name,
race, class, and randomized initial character state drawn from the bundled
ruleset. A generated character SHALL have no online metadata, passkey, or
network-derived data. Generation SHALL use fresh local randomness; it is not
required to reproduce browser character-creation RNG results, except that
initial prime stats SHALL use the desktop New Guy `3 + 3d6` algorithm where
each die is an integer in the inclusive range `0..5`.

#### Scenario: Generating an offline character

- **WHEN** the user supplies or confirms a name, race, and class
- **THEN** the system produces a valid level-one character using the selected
  traits, randomized initial state, and no online metadata

#### Scenario: Rejecting an unsupported trait selection

- **WHEN** the user supplies a race or class not present in the bundled ruleset
- **THEN** the system reports the invalid selection and creates no character

### Requirement: Desktop-compatible stat rolls

The wizard SHALL display the six rolled prime-stat values, their pre-bonus
total, and a total-quality indicator. The indicator SHALL use the desktop New
Guy color thresholds: dark gray below 46, gray from 46 through 54, white from
55 through 72, yellow from 73 through 80, and red above 80. Race and class
bonuses SHALL remain separate from the displayed roll total.

#### Scenario: Displaying a high-quality roll

- **WHEN** the six rolled prime stats total 81 or more before bonuses
- **THEN** the wizard displays the total with a red quality indicator

#### Scenario: Displaying a yellow roll

- **WHEN** the six rolled prime stats total from 73 through 80 before bonuses
- **THEN** the wizard displays the total with a yellow quality indicator

### Requirement: Validated character names

The system SHALL accept a character name of at most 30 Unicode scalar
characters, matching the desktop New Guy form's maximum length. It SHALL
reject empty names, whitespace-only names, and names containing Unicode
control characters. Non-control Unicode characters, including whitespace
between non-whitespace characters, SHALL be accepted. The system SHALL apply
the same validation to generated, interactive, and explicit CLI names before
creating or registering a character.

#### Scenario: Creating a character with a valid name

- **WHEN** the user supplies a name of 30 or fewer non-control characters
  containing at least one non-whitespace character
- **THEN** the system accepts the name for generation and registration

#### Scenario: Rejecting invalid names

- **WHEN** the user supplies an empty, whitespace-only, overlong, or
  control-character-containing name
- **THEN** the system reports the validation failure
- **AND** it creates and registers no character

### Requirement: Interactive New Guy creation

The system SHALL provide an interactive terminal creation flow when the
`new-guy` command is invoked without explicit creation inputs. The flow SHALL
provide navigable Name, Race, Class, and Stats rows. The focused row SHALL
determine the available editing actions and displayed key bindings. The flow
SHALL let the user edit or randomly replace the name, manually select the
initially randomized race and class, roll or unroll initial stats, confirm the
displayed character with “Sold!”, or cancel.

#### Scenario: Editing the focused name row

- **WHEN** the Name row is focused and the user enters printable characters or
  backspace
- **THEN** the wizard updates only the provisional name
- **AND** keys that operate other rows are accepted as name text

#### Scenario: Using row-specific controls

- **WHEN** the user focuses Race, Class, or Stats
- **THEN** the wizard displays only that row's applicable controls alongside
  global Sold! and cancel controls
- **AND** race/class selection changes and Random/Reroll actions do not
  conflict with name editing

  #### Scenario: Randomizing the focused name

  - **WHEN** the Name row is focused and the user invokes Random Name
  - **THEN** the wizard replaces only the provisional name
  - **AND** it retains the selected race, class, and current stats roll

  #### Scenario: Rolling stats

  - **WHEN** the Stats row is focused and the user invokes Roll
  - **THEN** the wizard replaces initial stats while retaining its current name,
  race, and class selections
  - **AND** it records the preceding roll for later unrolling

  #### Scenario: Unrolling stats

  - **WHEN** the Stats row is focused and the user invokes Unroll after one or
    more rolls
  - **THEN** the wizard restores the immediately preceding stats roll
  - **AND** it continues to allow unrolling until the complete roll history is
    exhausted

  #### Scenario: Replaying a rolled result after unrolling

  - **WHEN** the user rolls, unrolls, and then rolls again without another
    randomness-consuming wizard action
  - **THEN** the second roll produces the same stats as the roll that was
    unrolled

#### Scenario: Cancelling creation

- **WHEN** the user cancels the interactive creation flow before confirming
- **THEN** the system restores the terminal and does not register or persist a
  character

  #### Scenario: Correcting an invalid interactive name

  - **WHEN** the user selects Sold! with an invalid provisional name
  - **THEN** the wizard displays the validation error and remains open
  - **AND** it does not register a character

### Requirement: Confirmed local registration

The system SHALL register a generated character in the existing local
managed-character store only after the user confirms it interactively or
supplies explicit non-interactive creation inputs. It SHALL report the
credential-safe managed identity after successful registration.

#### Scenario: Confirming an interactive character

- **WHEN** the user selects “Sold!” in the wizard
- **THEN** the system registers the displayed generated character and reports
  its managed identity

#### Scenario: Creating a scripted character

- **WHEN** the user invokes `new-guy` with explicit name, race, and class
  inputs
- **THEN** the system bypasses the interactive wizard, registers one generated
  offline character, and emits credential-safe output in the requested format

### Requirement: Offline creation boundary

The system SHALL NOT perform HTTP requests, create an online leaderboard
identity, construct or transmit leaderboard requests, or expose a passkey
while generating or registering an offline character.

#### Scenario: Creating a character while disconnected

- **WHEN** the user creates and registers an offline character without network
  connectivity
- **THEN** the operation completes using only local resources and the
  character remains offline-only
