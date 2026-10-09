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

### Requirement: Autostart cleanup during character deletion

Confirmed deletion of an inactive, unowned character SHALL remove its persistent
user-service startup registration before removing character data. Cleanup SHALL
NOT stop a worker or change other characters' startup preferences. Active-runtime
protection and cancellation SHALL remain unchanged. A missing service template
with no startup registration SHALL NOT prevent deletion.

If cleanup fails, the command SHALL report the failure and retain all character
data. If data removal fails after successful cleanup, the command SHALL retain
the character data according to the existing atomic-removal contract and
explicitly report that autostart was disabled; it SHALL NOT claim complete
deletion or silently restore enablement.

#### Scenario: Deleting an autostart-enabled inactive character

- **WHEN** a user confirms deletion of an inactive, unowned character with autostart enabled
- **THEN** its startup registration is removed before its managed data is deleted, leaving no enabled instance for the removed character

#### Scenario: Cancelling or refusing deletion

- **WHEN** deletion is cancelled or rejected because a worker owns the character
- **THEN** neither the character's data nor its autostart preference is changed

#### Scenario: Failed startup cleanup

- **WHEN** confirmed deletion cannot remove the character's startup registration
- **THEN** the command reports an actionable failure and retains the character data

#### Scenario: Failed data removal after cleanup

- **WHEN** startup cleanup succeeds but atomic data removal fails
- **THEN** the character remains registered and the error explicitly states that its autostart is now disabled

#### Scenario: Deletion without an installed template

- **WHEN** a user deletes an inactive character on a host with no service template or startup registration for it
- **THEN** absence of the optional service template does not prevent deletion
