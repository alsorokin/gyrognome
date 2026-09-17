## Why

General leaderboard reporting cannot safely support native character creation
until Gyrognome knows how the browser's `Sold!` enrollment exchange reserves a
name, supplies online credentials, and triggers the first report. A mistaken
implementation could persist an unusable local character, retry an ambiguous
reservation request, or expose a credential.

## What Changes

- Add a disposable-only browser conformance procedure for online character
  enrollment, including successful creation, duplicate-name rejection, initial
  report ordering, and transport failures with uncertain server outcome.
- Record credential-free creation observations and assertions that native
  enrollment must satisfy before a later production transport change.
- Keep the normal CLI, local runtime, and offline New Guy flow transport-free;
  this change does not create online characters for normal users or transmit
  managed-character reports.

## Capabilities

### New Capabilities

- `online-enrollment-conformance`: Establishes safe, browser-derived evidence
  for server-authoritative character-name reservation and first-report
  activation using disposable characters only.

### Modified Capabilities

- None.

## Impact

- Affected systems: the Playwright conformance harness, credential-free
  fixtures/evidence validation, browser-protocol observations, and
  documentation.
- No production HTTP client, managed-character storage migration, normal CLI
  command, or live credential handling is added.
