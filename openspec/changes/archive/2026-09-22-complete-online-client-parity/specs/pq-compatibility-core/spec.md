# Spec Delta

## MODIFIED Requirements

### Requirement: Browser save interchange

The system SHALL import a browser `.pqw` export by Base64-decoding and parsing
its JSON document. It SHALL expose a typed canonical character state
containing traits, initial and current deterministic random state, attributes,
active and elapsed task state, progress bars, equipment, inventory, spells,
plots, quests, online metadata, and optional motto and guild profile values.
Missing motto and guild fields SHALL be interpreted as empty values because
the browser omits them until first assigned. It SHALL validate the nested type
and required content of each canonical field with field-specific errors. It
SHALL export a compatible Base64-encoded JSON document and preserve
unrecognized imported JSON data when no canonical state has been changed.

#### Scenario: Importing a valid browser save

- **WHEN** a user supplies a well-formed `.pqw` export from the browser client
- **THEN** the system makes its complete typed canonical character and online
  profile state available for inspection

#### Scenario: Importing optional profile fields

- **WHEN** a browser save contains string-valued motto or guild fields
- **THEN** the system exposes those values as credential-safe online profile
  state

#### Scenario: Importing a save before profile fields were assigned

- **WHEN** a valid browser save omits motto or guild
- **THEN** the system imports it with an empty value for each omitted field

#### Scenario: Rejecting an invalid save without loss

- **WHEN** a user supplies text that is not a valid Base64-encoded JSON save or
  has an invalid required nested canonical field or an invalid present profile
  field
- **THEN** the system reports a field-specific import error and does not create
  or modify character state

#### Scenario: Round-tripping unmodified imported data

- **WHEN** the system exports a successfully imported save without modifying
  canonical state
- **THEN** re-importing the export retains the imported canonical state,
  online profile values, and unrecognized JSON data

### Requirement: Safe character inspection and reference data

The system SHALL provide a read-only character inspection interface with a
human-readable character sheet and a machine-readable JSON representation of
canonical state. Both representations SHALL include traits, attributes, active
and elapsed task state, progress bars, equipment, inventory, spells, plots,
quests, online realm metadata, and credential-safe motto and guild values. The
interface SHALL redact online passkeys and omit unrecognized raw JSON data by
default. Committed reference fixtures and diagnostic output SHALL NOT contain
real passkeys, full signed request URLs, browser profiles, or player save
exports.

#### Scenario: Inspecting an online character safely

- **WHEN** a user inspects an imported online character with the default format
- **THEN** the character sheet identifies its realm and complete
  credential-safe state, including motto and guild when present, while
  redacting its passkey

#### Scenario: Producing machine-readable inspection output

- **WHEN** a user requests JSON inspection output for an imported character
- **THEN** the system produces valid JSON for its canonical and online profile
  state without an online passkey or unrecognized raw JSON fields

#### Scenario: Committing compatibility fixtures

- **WHEN** browser observations are converted into project fixtures
- **THEN** the fixtures use synthetic credentials and contain no
  player-identifying save data

