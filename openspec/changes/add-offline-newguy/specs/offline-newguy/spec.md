## Purpose

Create and locally register offline-only Progress Quest characters through a
browser-like terminal creation experience or explicit command-line inputs.

## ADDED Requirements

### Requirement: Offline character generation

The system SHALL generate a valid level-one canonical character from a name,
race, class, and randomized initial character state drawn from the bundled
ruleset. A generated character SHALL have no online metadata, passkey, or
network-derived data. Generation SHALL use fresh local randomness; it is not
required to reproduce browser character-creation RNG results.

#### Scenario: Generating an offline character

- **WHEN** the user supplies or confirms a name, race, and class
- **THEN** the system produces a valid level-one character using the selected
  traits, randomized initial state, and no online metadata

#### Scenario: Rejecting an unsupported trait selection

- **WHEN** the user supplies a race or class not present in the bundled ruleset
- **THEN** the system reports the invalid selection and creates no character

### Requirement: Interactive New Guy creation

The system SHALL provide an interactive terminal creation flow when the
`new-guy` command is invoked without explicit creation inputs. The flow SHALL
let the user edit or randomize the name, select or randomize the race and
class, reroll only initial stats, confirm the displayed character with
“Sold!”, or cancel.

#### Scenario: Randomizing all selections

- **WHEN** the user invokes the wizard's randomize action
- **THEN** the wizard replaces the name, race, class, and initial stats with a
  newly generated combination

#### Scenario: Rerolling stats

- **WHEN** the user invokes the wizard's reroll action
- **THEN** the wizard replaces initial stats while retaining its current name,
  race, and class selections

#### Scenario: Cancelling creation

- **WHEN** the user cancels the interactive creation flow before confirming
- **THEN** the system restores the terminal and does not register or persist a
  character

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
