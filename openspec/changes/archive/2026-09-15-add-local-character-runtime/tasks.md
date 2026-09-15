## 1. Local storage foundation

- [x] 1.1 Add the SQLite, XDG data-directory, stable identifier, and advisory-lock dependencies; verify `cargo check` succeeds with the new dependency graph.
- [x] 1.2 Implement data-root resolution and schema initialization for managed characters, including versioned canonical-state and original-document storage; verify temporary-directory tests cover XDG override and first-run database creation.
- [x] 1.3 Implement transactional managed-character registration from a validated browser save and credential-safe character lookup; verify tests cover stable identifiers, inspection data, and no record after invalid imports.
- [x] 1.4 Implement atomic canonical-state updates and reads that preserve the last complete state on a failed transaction; verify persistence tests simulate a failed update and a reopened database.

## 2. Exclusive runtime worker

- [x] 2.1 Implement non-blocking, per-character advisory lock acquisition with errors that identify an already-owned character; verify concurrent-worker tests reject the second owner and allow ownership after the first exits.
- [x] 2.2 Implement the managed-character worker with a monotonic active-process clock, periodic explicit-duration calls to `simulation::advance`, and transactional persistence after each successful result; verify worker tests assert persisted progression and no downtime catch-up after restart.
- [x] 2.3 Make simulation and persistence failures terminate the worker without replacing the last good state; verify tests cover an unsupported simulation transition and an injected storage failure.

## 3. User-service lifecycle and CLI

- [x] 3.1 Add CLI operations to register a character, inspect managed-character state, and run a worker for a supplied identifier; verify CLI integration tests cover valid registration, missing identifiers, and redacted output.
- [x] 3.2 Package a `gyrognome@.service` systemd user template and implement validated `systemctl --user` start, stop, status, and recovery commands; verify command-runner tests cover active, inactive, unavailable-manager, and service-failure outcomes.
- [x] 3.3 Ensure stop handling completes no partial state update and status reports persisted identity plus runtime ownership; verify integration tests stop a running worker and assert a readable final persisted state.

## 4. Safety, documentation, and validation

- [x] 4.1 Add fixture-safety and dependency-boundary coverage confirming runtime output redacts passkeys and the crate has no HTTP transport dependency or execution path; verify `cargo test --test fixture_safety` and the new runtime-boundary tests pass.
- [x] 4.2 Document local data location, character registration, lifecycle commands, no-downtime-catch-up behavior, systemd user-service prerequisite, recovery behavior, and continued offline-only boundary; verify documented commands against synthetic fixtures.
- [x] 4.3 Run formatting, warning-denied linting, targeted runtime tests, fixture-safety tests, and the complete test suite; verify all commands complete without warnings or failures.
