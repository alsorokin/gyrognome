# Spec Delta

## ADDED Requirements

### Requirement: Case-insensitive dashboard letter shortcuts

The dashboard and its character selector SHALL recognize both ASCII cases of each existing letter shortcut, including Ctrl+C, regardless of Caps Lock. Case-insensitive matching SHALL preserve existing action eligibility, confirmation requirements, modifier handling, and key-event-kind handling. Non-letter controls SHALL retain their existing behavior.

#### Scenario: Dashboard actions with Caps Lock enabled

- **WHEN** the dashboard receives `Q`, `B`, `M`, `G`, `S`, or `A` outside a profile editor
- **THEN** it handles the input as the corresponding lowercase quit, brag, motto, guild, lifecycle, or autostart shortcut
- **AND** existing eligibility checks and confirmation requirements still apply

#### Scenario: Selecting a character with uppercase shortcuts

- **WHEN** the character selector receives `J`, `K`, or `Q`
- **THEN** it performs the same next, previous, or cancel action as `j`, `k`, or `q`, respectively

#### Scenario: Interrupting with an uppercase control character

- **WHEN** the dashboard, character selector, or profile editor receives Ctrl+C represented with uppercase `C`
- **THEN** it performs the same quit or cancel behavior as Ctrl+C represented with lowercase `c`
- **AND** an open profile editor does not submit its contents

#### Scenario: Keeping non-letter controls unchanged

- **WHEN** Caps Lock is enabled and the user invokes Enter, Escape, arrow keys, PageUp, PageDown, Tab, Shift+Tab, or F1 through F6 where supported
- **THEN** each control retains its existing action

### Requirement: Preserve profile input case

Motto and guild editors SHALL preserve the exact case of printable text input, including uppercase letters that are shortcuts outside the editor. Case-insensitive shortcut matching SHALL NOT convert editor text or execute dashboard letter actions while editing.

#### Scenario: Typing uppercase shortcut letters in a profile editor

- **WHEN** a user types `QBM GSA` into an open motto or guild editor
- **THEN** the text is inserted with that exact case
- **AND** no quit, brag, profile-opening, lifecycle, or autostart action is triggered

#### Scenario: Submitting mixed-case profile text

- **WHEN** a user enters valid mixed-case text in a motto or guild editor and submits with Enter
- **THEN** the submitted text retains its case and passes through existing validation and action handling
