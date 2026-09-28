## Why

Classic desktop guild requests can update the Spoltog leaderboard while
Gyrognome retains the previous local guild. The live evidence generator and
production classifier normalize response bodies with different replacement
values and different sensitive-field sets, so evidence-backed fingerprints
cannot reliably match in production.

## What Changes

- Define one shared desktop response-normalization and SHA-256 fingerprint
  contract used by both live evidence generation and production guild response
  classification.
- Make production classification supply the same ordered dynamic values used
  when the bundled evidence was generated, including identity, authentication,
  passkey, and prior/submitted guild values.
- Add cross-path contract tests so evidence generation and production
  classification cannot silently diverge again.
- Retain a bounded, credential-free public-profile check as a secondary
  reconciliation path when a guild response remains indeterminate, without
  repeating the guild mutation.
- Persist the server-observed canonical guild capitalization only when it
  matches the submitted ASCII designation case-insensitively.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `leaderboard-conformance`: Require the live evidence path and production
  classifier to use one versioned normalization/fingerprint contract with
  cross-path regression vectors.
- `online-character-profile`: Define evidence-backed guild persistence with a
  bounded public-profile reconciliation fallback and canonical designation
  handling.
- `opt-in-leaderboard-reporting`: Define the permitted credential-free
  verification request after an indeterminate explicit guild action.

## Impact

Affected code includes desktop live-conformance fingerprint generation,
desktop guild response classification, HTTPS public-profile retrieval,
reporting orchestration, and their Rust/Node tests. The bundled evidence format
may gain an explicit normalization identifier, but no credentials, raw
responses, signed URLs, or player saves will be persisted. No new dependency or
automatic background network synchronization is intended.
