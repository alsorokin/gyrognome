# Spec Delta

## ADDED Requirements

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
