## 1. Typed canonical state

- [x] 1.1 Define immutable typed structures for traits, Alea states, attributes, task and elapsed state, progress bars, equipment, inventory, spells, plots, quests, and non-secret online metadata; verify the model can represent the synthetic full-state fixture.
- [x] 1.2 Replace shallow top-level validation with field-specific nested parsing into the canonical model while retaining the raw document for unmodified export; verify malformed nested fixture variants identify the failing field.
- [x] 1.3 Preserve list ordering and browser field names in canonical state; verify parsed synthetic equipment, inventory, spells, plots, and quests retain their fixture order and values.

## 2. Inspection formats

- [x] 2.1 Expand default `inspect` output into grouped text sections for identity, attributes, current activity, progress, equipment, inventory, spells, plots, quests, and online realm; verify snapshot-style CLI tests cover the complete synthetic character sheet.
- [x] 2.2 Add an explicit JSON inspection format containing the canonical state with stable field and collection ordering; verify it parses as JSON and matches the synthetic fixture's canonical values.
- [x] 2.3 Enforce passkey and raw-document exclusion in text and JSON output; verify output regression tests reject both a synthetic passkey and an unrecognized raw fixture field.

## 3. Compatibility evidence

- [x] 3.1 Add a synthetic full-state browser-save fixture plus valid and malformed nested variants without player data; verify all committed fixtures pass the existing fixture-safety checks.
- [x] 3.2 Manually inspect both ignored disposable browser saves in text and JSON formats; verify their reported state is complete, fields are coherent, and no passkey is displayed.
- [x] 3.3 Retain unmodified export round-trip coverage after introducing the canonical model; verify exports preserve unrecognized source data while inspection excludes it.

## 4. Documentation and validation

- [x] 4.1 Document the complete inspection views, JSON format, redaction guarantees, and continued read-only boundary; verify documented commands run against synthetic fixtures.
- [x] 4.2 Run formatting, linting, and the complete test suite; verify the project reports no warnings or failures.
