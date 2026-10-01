# Proposal

## Why

Pemptus desktop saves already use the supported desktop-6.4.4 simulation, but
production reporting and profile actions are restricted to Spoltog's
account/password contract. Source inspection and the archived manual probe
establish a concrete passkey-only integration path, while full operation
acceptance and anti-cheat conformance still require separate live evidence.

## What Changes

- Add exact realm/endpoint/authentication contracts for Spoltog account/password
  and Pemptus passkey-only characters, preserving the existing desktop profile,
  source-derived rules, and browser behavior.
- Support Pemptus HTTPS delivery without an Authorization header, retaining
  Spoltog Basic authentication and fail-closed redirect/destination handling.
- Select and validate evidence for the requested realm, credential mode, import
  path, and operation; remove the requirement that unrelated guild or automatic
  operation evidence must pass before an explicit action can be eligible.
- Wire the same eligibility decisions through save inspection, managed storage,
  worker reports, manual brag, motto/guild actions, and dashboard presentation.
  Show partial availability honestly rather than labeling every operation
  unavailable when only one is gated.
- Extend the development-only conformance workflow for separately approved
  Pemptus immediate operations and real-time level/act progression, with
  observable acceptance checks, one-shot intents, and private-artifact cleanup.
- Permit an explicitly approved older disposable Pemptus character for the
  immediate stage only, retaining fresh level-1 handoff checks and requiring
  exact public/saved baseline agreement. An approved empty baseline motto and
  one separately counted preparatory guild leave can avoid official-client
  mutations. Preserve exploratory originals and clean up only private copies;
  this exception does not authorize progression or weaken acceptance evidence.
- Activate only operations covered by validated, credential-free Pemptus
  evidence. Missing or inconclusive evidence keeps the corresponding gate
  closed; the earlier HTTP-200 probe is not production-enabling evidence.
- Preserve atomic local-only provenance, fresh-import recovery, existing
  spelling-normalization equivalence, and rejection of substantive adaptations.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `opt-in-leaderboard-reporting`: Pemptus passkey-only delivery and independent
  realm/operation eligibility for managed reporting.
- `online-character-profile`: Pemptus motto/guild availability with matching
  authentication and realm-specific guild response evidence.
- `leaderboard-conformance`: Independently validated operation evidence and
  staged Pemptus conformance distinct from the archived diagnostic.
- `terminal-dashboard`: Accurate mixed-operation availability and action gating.

## Impact

Desktop eligibility, evidence, transport, profile classification, reporting,
save inspection, runtime presentation, and dashboard modules; feature-gated
live tooling; synthetic conformance/regression tests and credential-free
evidence fixtures; README and desktop compatibility documentation.

No new external dependencies or database migration are expected. Existing
Spoltog evidence must remain usable only where its conformance-relevant
behavior is demonstrably unchanged; affected contracts require new evidence.
Live experimentation needs fresh explicit approval of identities, operations,
attempt bounds, progression duration, and cleanup. This proposal authorizes
none of those network mutations.

## Non-goals

Native desktop enrollment or character creation, other realms or endpoint
aliases, Pemptus account/password support, non-ASCII desktop encoding, new
simulation rules, recovery of historical RNG, queued/retried reports, or
reconnecting an already local-only managed timeline.
