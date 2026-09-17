# managed-character-administration Specification

## Purpose

Provide credential-safe local administration of registered characters without
requiring users to inspect or modify the managed-character database directly.

## Requirements

### Requirement: Confirmed managed-character deletion

The system SHALL provide a command that deletes a registered managed character
by its stable identifier. It SHALL display the credential-safe identity of the
target and require an explicit interactive confirmation before deleting any
persisted character data. Declining confirmation SHALL leave the character
unchanged. The command SHALL report an actionable error when the identifier is
malformed or not registered.

#### Scenario: Deleting a stopped managed character

- **WHEN** a user confirms deletion of a registered character whose runtime is
  inactive
- **THEN** the system removes that character's persisted managed data and
  reports its successful deletion without exposing save credentials

#### Scenario: Cancelling character deletion

- **WHEN** a user declines the deletion confirmation
- **THEN** the system leaves the selected managed character registered and
  reports that deletion was cancelled

### Requirement: Active runtime protection

The system SHALL refuse to delete a managed character while its local runtime
is active or owns the character. It SHALL leave all persisted character data
intact and instruct the user to stop the runtime before retrying deletion.

#### Scenario: Attempting to delete a running character

- **WHEN** a user confirms deletion of a character with an active local runtime
- **THEN** the system does not delete its data and reports that the runtime must
  be stopped first

### Requirement: Credential-safe character administration

Character-administration interactions and output SHALL expose only
credential-safe character identity and local management status. They SHALL NOT
display browser passkeys, raw save documents, or unrecognized raw save fields,
and SHALL NOT make network requests.

#### Scenario: Deleting an online-originated character

- **WHEN** a user administers a character registered from a browser save
- **THEN** the system performs the local action without displaying its passkey
  or contacting a remote service
