## 1. Reporting Boundary

- [x] 1.1 Change foreground manual-brag target acquisition to use the
  per-character online-action lock without acquiring worker ownership, and
  verify reporting unit tests can submit while a `Worker` owns the character.
- [x] 1.2 Remove or consolidate the obsolete inactive-only reporting target
  and optional ownership-lock state while preserving the worker-specific
  automatic report path, and verify runtime target tests cover active,
  inactive, offline, unknown, malformed, and missing-credential cases.
- [x] 1.3 Preserve conformance validation, official-endpoint validation,
  persisted snapshot construction, one-attempt delivery, and credential-safe
  outcomes, and verify the existing reporting safety and outcome tests pass.

## 2. Active-Runtime Regression Coverage

- [x] 2.1 Add a reporting regression that holds an active worker, delivers
  exactly one manual brag from the current persisted character/profile
  snapshot, and verifies worker ownership and managed state remain intact.
- [x] 2.2 Add or extend CLI integration coverage for a confirmed `report`
  command while the character is active, verifying successful safe output,
  exactly one test transport action, and no credential or raw-save disclosure.
- [x] 2.3 Retain dashboard Brag coverage for one immediate shared-provider call
  and categorized safe outcome messages, adding local-provider coverage if
  needed to verify the active-runtime path no longer returns `already running`.

## 3. Validation

- [x] 3.1 Run focused Rust tests for reporting, runtime boundaries, dashboard
  actions, and CLI report behavior through Ubuntu WSL and verify all selected
  tests pass.
- [x] 3.2 Synchronize the Windows checkout to the disposable ext4 validation
  mirror, run the full Rust and Node.js test suites there, and verify both
  suites pass without modifying the mirror directly.
