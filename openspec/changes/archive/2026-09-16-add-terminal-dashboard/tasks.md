## 1. Dashboard foundation and safe presentation

- [x] 1.1 Add `ratatui` and `crossterm` dependencies and introduce dashboard module boundaries that depend only on credential-safe runtime types; verify `cargo check` succeeds and the dependency-boundary test rejects raw save-document access.
- [x] 1.2 Implement dashboard snapshot collection for one managed character, including canonical state, service status, and non-fatal status messages; verify fake-provider tests cover registered, missing, active, inactive, and failed-service snapshots.
- [x] 1.3 Implement responsive dashboard rendering for identity, activity, progress bars, equipment, inventory, spells, plot, quests, service status, and visible keyboard help; verify headless rendering tests cover complete, compact, and too-small terminal layouts.
- [x] 1.4 Enforce presentation redaction so no rendered state, action feedback, or diagnostics includes passkeys, raw documents, or unrecognized source fields; verify regression tests use an online-originated synthetic character.

## 2. Interaction and lifecycle control

- [x] 2.1 Add `gyrognome dashboard <character-id>` with a bounded configurable refresh interval and validate the identifier before entering terminal mode; verify CLI tests reject an unknown character without enabling interactive mode.
- [x] 2.2 Implement polling and explicit-refresh events that reload persisted state and service status without calling simulation, worker, lock-acquisition, or state-update APIs; verify controller tests observe newer provider snapshots while recording no mutating calls.
- [x] 2.3 Implement documented `q`, `r`, `s`, `x`, and `c` key bindings and a confirmation state for start, stop, and recover; verify event tests cover confirmation, cancellation, action dispatch, and refreshed result feedback.
- [x] 2.4 Route confirmed lifecycle actions through the existing lifecycle interface and render success or actionable failure without exiting the dashboard; verify fake-lifecycle tests cover unavailable user managers, failed starts, and preserved last successful snapshots.

## 3. Terminal lifecycle and delivery

- [x] 3.1 Implement an RAII terminal-session guard for raw mode and alternate-screen entry, including partial-setup and error-path cleanup; verify terminal-backend tests assert restoration after quit, render failure, input failure, and lifecycle failure.
- [x] 3.2 Wire interrupt handling to the dashboard event loop so terminal interrupts exit through the same restoration path; verify an injected interrupt produces a clean successful dashboard exit.
- [x] 3.3 Document dashboard startup, key bindings, refresh behavior, lifecycle confirmations, systemd prerequisites, redaction guarantees, and observer-only/no-network boundaries; verify documented commands against an isolated synthetic runtime data directory.
- [x] 3.4 Run formatting, warning-denied linting, dashboard unit and integration tests, fixture-safety tests, and the complete test suite; verify all commands complete without warnings or failures.
