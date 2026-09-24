## MODIFIED Requirements

### Requirement: Safe character inspection and reference data

The system SHALL provide a read-only character inspection interface with a
human-readable character sheet and a machine-readable JSON representation of
canonical state. Both representations SHALL include traits, attributes, active
and available elapsed task state, progress bars, equipment, inventory, spells,
plots, quests, online realm metadata, and credential-safe motto and guild
values. They SHALL identify the source format, continuation profile, unavailable
desktop history, and reporting eligibility without claiming an unproven writer
version. The interface SHALL redact online passkeys and omit account logins,
passwords, authenticated endpoint URLs, unrecognized raw JSON data, and raw
desktop properties by default. Committed reference fixtures and diagnostic
output SHALL NOT contain real credentials, full signed/authenticated request
URLs, browser profiles, or player save exports of any supported format.

#### Scenario: Inspecting an online character safely

- **WHEN** a user inspects an imported online character with the default format
- **THEN** the character sheet identifies its realm and complete
  credential-safe state, including motto and guild when present, while
  redacting its passkey and excluding account credentials

#### Scenario: Producing machine-readable inspection output

- **WHEN** a user requests JSON inspection output for an imported character
- **THEN** the system produces valid JSON for its canonical and online profile
  state without credentials or unrecognized raw save fields

#### Scenario: Committing compatibility fixtures

- **WHEN** browser or desktop observations are converted into project fixtures
- **THEN** the fixtures use synthetic credentials and contain no
  player-identifying save data

#### Scenario: Inspecting desktop provenance

- **WHEN** a user inspects a supported desktop save
- **THEN** output identifies desktop-6.4.4 continuation separately from source
  provenance, marks unavailable history, and explains any online ineligibility
  without a network request
