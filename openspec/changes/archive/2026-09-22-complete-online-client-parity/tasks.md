# Tasks

## 1. Persist Online Profile State

- [x] 1.1 Add credential-safe motto and guild profile types, parse optional string values from browser saves with empty defaults, and include them in safe human-readable and JSON inspection; verify focused save/state tests cover present, omitted, and invalid fields without exposing passkeys or unknown source data.
- [x] 1.2 Advance the managed SQLite schema with non-null motto and guild columns, transactionally backfill valid optional values from retained documents, and preserve canonical state and credentials; verify migration tests open a version-1 database, retain existing data, import profile values safely, and continue rejecting newer schemas.
- [x] 1.3 Add independent profile read/update store operations and include profile metadata in managed-character reads; verify transaction and concurrent-store tests demonstrate profile updates cannot overwrite worker-persisted simulation state and failed updates retain the previous complete profile.

## 2. Serialize and Deliver Online Actions

- [x] 2.1 Add a short-lived per-character online-action advisory lock distinct from the worker ownership lock and use it for manual brags and worker-generated reports; verify concurrency tests establish exclusive request ordering without preventing an active worker or profile update.
- [x] 2.2 Replace empty worker/manual report mottos with the current persisted motto while preserving transition snapshots and Specialty refresh behavior; verify reporting and runtime tests assert level-up, act-completion, and manual-brag requests carry the expected motto.
- [x] 2.3 Implement validated motto set/clear operations that persist before one `t=m` delivery, retain the value on rejected or failed delivery, and reject offline characters, unofficial endpoints, invalid credentials, control characters, or failed evidence before transport; verify focused reporting and storage tests cover each outcome and redaction invariant.
- [x] 2.4 Extend the transport with bounded guild-response reading, prior/submitted-designation normalization, and sanitized accepted, rejected, and indeterminate fingerprint categories, never returning or logging the raw body or normalized text; verify parser tests cover every conformance-derived accepted and rejected fingerprint, oversized bodies, unknown content, HTTP rejection, and delivery failure.
- [x] 2.5 Implement guild-designation submission so non-empty values join or change guilds and an empty value leaves, sending one browser-compatible request and persisting the submitted value only after a conformance-recognized acceptance; verify tests cover active and inactive characters, accepted non-empty and empty submissions, preserved prior membership on every failure, official-endpoint restriction, and no retry.

## 3. Establish Guild Conformance Evidence

- [x] 3.1 Extend the disposable browser harness with credential-free accepted non-empty, deliberately invalid, and accepted empty guild-designation submissions, ephemeral response normalization, distinct safe fingerprints, explicit cleanup verification, and sanitized request descriptors; verify the Node test suite covers dry-run interception, ordering, cancellation/failure handling, accepted/rejected fingerprint classification, and rejection of sensitive evidence.
- [x] 3.2 Extend bundled evidence validation to require passing accepted non-empty, rejected invalid, accepted empty, distinct-fingerprint, and cleanup observations while retaining all existing enrollment, trace, and anti-cheat requirements; verify JavaScript and Rust fixture-validation tests fail closed for each missing, malformed, ambiguous, non-passing, or sensitive guild field.
- [x] 3.3 Run the explicitly confirmed disposable live conformance procedure against a testable existing guild plus a deliberately invalid designation, verify the disposable character returns to no guild and remains normally classified, then commit only the sanitized passing fingerprints and evidence required by the validator.
- [x] 3.4 Gate production guild actions on the updated bundled evidence and verify no transport call occurs when guild conformance is unavailable or invalid.

## 4. Add Command-Line Profile Actions

- [x] 4.1 Add `motto <id> <text>` with `--clear` and `guild <id> <designation>`, where an explicitly supplied empty designation leaves the current guild, including argument validation and credential-safe success/failure output; verify CLI parser and integration tests cover motto set/clear, non-empty/empty guild submissions, malformed identifiers, invalid input, offline characters, active workers, and safe output.
- [x] 4.2 Ensure CLI profile actions invoke exactly one online operation without interactive confirmation beyond the explicit command and arguments; verify integration transports record one request and cancellations or argument errors record none.

## 5. Add Dashboard Profile Editing

- [x] 5.1 Extend dashboard snapshots and the provider boundary with profile values and motto/guild operations; verify fake-provider tests record inputs and categorized outcomes while refreshes and lifecycle actions remain network-free.
- [x] 5.2 Add `m` and `g` modal editor states initialized from persisted values, with printable Unicode input, Backspace editing, Enter submission, and Escape cancellation; submitting an empty guild value leaves the guild, with no separate leave action. Verify event-mapping and state-machine tests cover editing action-key characters, motto clearing, empty guild submission, cancellation, active-runtime operation, and error messages.
- [x] 5.3 Refresh persisted profile state after dashboard actions and render only credential-safe categorized outcomes; verify tests show updated values after success, retained guild after rejection, retained motto after delivery failure, and no raw endpoint body or credentials.

## 6. Refine Dashboard Presentation

- [x] 6.1 Remove Quest target from Details, render Motto and Guild independently only when each value is non-empty, and let expanded Details fill the remaining left-column height with a six-row minimum when space permits while reducing Activity to four; verify content, remaining-height allocation, and two-row collapsed Details across short/tall layouts and all pane-collapse combinations.
- [x] 6.2 Cap expanded Equipment at thirteen total rows so its inner content never exceeds the eleven equipment slots while preserving collapse behavior and useful allocation of remaining space; verify layout tests cover short and tall full-layout terminals plus all collapsed-pane combinations affected by the cap.
- [x] 6.3 Add a presentation-only elapsed-duration formatter that emits compact day/hour/minute/second components, omits leading zero units, and renders zero as `0s`; verify unit and render tests cover boundary values at 0, 59, 60, 3600, 86400, and 90061 seconds.
- [x] 6.4 Update dashboard keyboard help, narrow/full rendering, and snapshots for the new editors and details content; verify the targeted dashboard test suite passes without changing compact progress prediction behavior.

## 7. Document and Validate the Completed Client Surface

- [x] 7.1 Update README support, CLI, dashboard, reporting safety, persistence, and conformance sections for motto and guild operations plus the revised dashboard details; verify documented commands and keys match CLI help and rendered footer text.
- [x] 7.2 Run formatting and the smallest focused Rust and Node test selections covering state, runtime, reporting, CLI integration, dashboard, fixtures, and leaderboard conformance; fix only failures caused by this change.
- [x] 7.3 Run the complete Rust and Node test suites and verify all OpenSpec requirements are represented by passing automated tests or the sanitized live guild-conformance evidence.
