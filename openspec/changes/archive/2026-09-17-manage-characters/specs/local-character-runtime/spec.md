## ADDED Requirements

### Requirement: Safe managed-character removal

The local managed-character store SHALL remove every persisted record and
runtime metadata associated with a registered character when requested through
the character-administration interface. It SHALL reject removal of an unknown
character and SHALL preserve the character when removal cannot complete
atomically. It SHALL not remove a character while a local runtime owns it.

#### Scenario: Removing a registered inactive character

- **WHEN** character administration requests removal of a registered character
  that has no active runtime owner
- **THEN** the store removes the character's persisted state, original save
  document, and runtime metadata as one complete removal

#### Scenario: Failing a character removal

- **WHEN** character removal encounters a storage failure before completion
- **THEN** the system reports the failure and retains a complete readable
  managed character rather than a partial record
