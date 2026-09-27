## Why

Gyrognome cannot import saves from the original Windows Progress Quest client.
Converting those saves into the existing browser simulation would change
leaderboard-visible progression, so desktop imports need their own verified
continuation and reporting contract, not just a binary decoder.

## What Changes

- Add bounded, read-only inspection and registration of the observed 6.2 and
  6.4.4 `.pq` format, including same-format `.bak` files. Decode zlib-compressed
  Delphi component streams without loading executable components.
- Persist explicit browser or desktop-6.4.4 compatibility identity, private
  desktop credentials, import provenance, and resumable profile-specific state.
  Existing browser saves, managed characters, enrollment, and conformance remain
  unchanged.
- Continue supported desktop imports using pinned 6.4.4 rules, integer rewards,
  desktop random primitives, callback timing, and report-boundary snapshots.
  Treat 6.2-to-6.4.4 adaptation as an explicitly evidenced migration; reject
  unsupported ambiguous states rather than silently using browser behavior.
- Establish source-derived desktop conformance fixtures from the pinned 6.4.4
  source before implementing production continuation. Use synthetic
  cross-implementation differential tests, and treat separately approved,
  network-blocked execution of the verified official executable as optional
  corroboration rather than a prerequisite for local support.
- Add revision-8 request construction and allowlisted classic-realm delivery
  with private account authentication where required. Keep each desktop online
  operation disabled until matching local and separately approved disposable
  live conformance evidence passes. Permit a source-derived deterministic load
  normalization, such as the desktop spelling corrections, to share that
  operation scope only when equivalence tests prove it produces the same
  canonical post-load and protocol state. Substantive legacy migrations remain
  separately gated. Alpaquil evidence cannot enable classic reporting.
- Allow local advancement while classic reporting is gated. Once advanced in
  that state, an online-originated import remains local-only; later online use
  requires a fresh official-client import rather than accumulated reporting.
- Expose safe compatibility and reporting-eligibility information through the
  existing CLI/dashboard surfaces. Never reveal account names, passwords,
  passkeys, authenticated URLs, or raw desktop saves.

No source-save mutation, desktop save export, native classic account creation,
exact recovery of an unsaved historical RNG state, or separate 6.2 simulation
engine is proposed. Creating this plan does not authorize launching a client,
creating an online identity, or sending reports.

## Capabilities

### New Capabilities

- `desktop-save-compatibility`: Safe desktop decoding, 6.4.4 continuation,
  evidenced legacy adaptation, desktop request construction, and private
  credential handling.

### Modified Capabilities

- `pq-compatibility-core`: Extend safe inspection/reference-data boundaries to
  desktop saves while preserving browser interchange and request primitives.
- `deterministic-simulation`: Dispatch by persisted compatibility profile;
  retain the browser contract and add a distinct desktop callback contract.
- `local-character-runtime`: Atomically register and resume desktop state,
  schedule profile-specific advancement, and gate desktop online delivery.
- `leaderboard-conformance`: Add source-derived desktop local evidence and
  separate live evidence scoped to source identity, migration path, realm,
  authentication, and operation.
- `opt-in-leaderboard-reporting`: Select protocol and eligibility by profile,
  retaining one-shot delivery, confirmation, and online-action serialization.
- `online-character-profile`: Import desktop motto/guild values and apply
  profile-specific, credential-safe motto and guild operations.
- `terminal-dashboard`: Display compatibility/eligibility safely and preserve
  read-only presentation and action gating for desktop characters.

## Impact

The main integration points are `src/save.rs`, `state.rs`, `rng.rs`,
`ruleset.rs`, `simulation.rs`, `runtime.rs`, `protocol.rs`, `reporting.rs`,
`fixtures.rs`, `cli.rs`, and `dashboard.rs`. Database/canonical-state migration
must preserve older browser records and private original JSON.

Desktop decoding needs a bounded zlib dependency and a narrowly scoped Delphi
stream/list-record reader. A separate developer-only desktop oracle and
sanitized fixtures complement, rather than replace, the existing browser
Playwright harness. The local oracle may be source-derived; optional execution
of the verified official executable can corroborate selected behavior but does
not block local implementation. README/help, fixture-safety checks,
simulation/runtime tests, and reporting/profile tests must cover the new
profile and disclose unverified Delphi numeric/random edge equivalence.

Classic server acceptance, account authentication over HTTPS, Delphi runtime
numeric edge equivalence, and difficult legacy transitions remain explicit
limitations or evidence milestones, not assumptions that may be marked
passing. Production desktop reporting stays closed when a required live
milestone or approval is unavailable.
