# Spec Delta

## MODIFIED Requirements

### Requirement: Interactive New Guy creation

The system SHALL provide an interactive terminal creation flow when the
`new-guy` command is invoked without explicit creation inputs. The flow SHALL
provide navigable Mode, Name, Race, Class, and Stats rows. The focused row
SHALL determine the available editing actions and displayed key bindings. When
Offline is selected in the Mode row, the flow SHALL let the user edit or
randomly replace the name, manually select the initially randomized race and
class, roll or unroll initial stats, confirm the displayed offline character
with “Sold!”, or cancel.

#### Scenario: Editing the focused name row

- **WHEN** the Name row is focused and the user enters printable characters or
  backspace
- **THEN** the wizard updates only the provisional name
- **AND** keys that operate other rows are accepted as name text

#### Scenario: Using row-specific controls

- **WHEN** the user focuses Mode, Race, Class, or Stats
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

#### Scenario: Selecting offline creation

- **WHEN** a user selects Offline in the Mode row
- **THEN** the system opens the offline-only draft and does not perform online
  enrollment
