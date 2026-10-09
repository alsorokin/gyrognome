## ADDED Requirements

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
