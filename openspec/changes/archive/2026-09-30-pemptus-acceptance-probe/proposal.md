# Proposal

## Why

Pemptus desktop saves already import under the shared desktop-6.4.4 simulation,
but the existing live experiment supports only Spoltog account/password
authentication. A narrowly scoped native probe is needed to observe Pemptus
manual-report delivery and public classification without enabling production
reporting or conducting the full conformance experiment.

## What Changes

- Add a development-only, feature-gated Pemptus manual-brag probe using the
  existing desktop importer and Rust revision-8 request constructor.
- Require explicit disposable-character, stopped-official-client, and live
  submission confirmations; send at most one request per approved execution.
- Accept only the exact Pemptus saved endpoint and passkey-only authentication,
  using the fixed official HTTPS endpoint without an Authorization header.
- Observe the public character row before delivery and for at most 60 seconds
  afterward, reporting delivery and classification separately.
- Keep credentials and signed requests in memory, preserve source saves, and
  perform no progression, managed registration, motto/guild changes, or retries.
- Document the diagnostic's limits, particularly when the public state already
  matches the submitted state.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `leaderboard-conformance`: Add a bounded diagnostic probe distinct from
  production-enabling classic live conformance evidence.

## Impact

Feature-gated development binaries and desktop experiment helpers, nearby Rust
tests, and `docs/classic-desktop-compatibility.md`. The native protocol and
importer are reused; simulation, managed reporting eligibility, bundled Spoltog
evidence, and normal CLI behavior remain unchanged. Execution mutates only the
explicitly approved disposable character's Pemptus manual-report state.

## Non-goals

Full Pemptus online support, enrollment, automatic progression, motto/guild
experiments, production evidence generation, or proof of unpublished
server-side anti-cheat rules. Real saves and credentials must not become test
fixtures or committed artifacts.
