## 1. Pure report-event tracing

- [x] 1.1 Define credential-free report-event and transition-snapshot types for `s`, `l`, `a`, `b`, and `m`; verify serialization cannot contain a passkey or signed request URL.
- [x] 1.2 Extend deterministic advancement to produce ordered `l` and `a` snapshots at their browser call sites while preserving its existing final state; verify state equivalence with the current checkpoint suite.
- [x] 1.3 Add explicit initial-load, manual-brag, and motto-change trace inputs; verify their triggers and snapshot timing against browser observations.
- [x] 1.4 Add unit coverage for multiple report-producing transitions in one advancement; verify event order and every intermediate snapshot exactly.

## 2. Browser trace and protocol conformance

- [x] 2.1 Capture sanitized, disposable-browser report traces for all five triggers, including level and act transitions; verify source revision and source hash are recorded without live credentials.
- [x] 2.2 Extend report construction to consume transition snapshots and expose unsigned fields, normalized representation, and validator output; verify every synthetic trace fixture replays exactly.
- [x] 2.3 Extend fixture-safety validation to report traces and experiment artifacts; verify it rejects player saves, live passkeys, browser profiles, and complete signed request URLs.
- [x] 2.4 Add regression tests proving trace capture and protocol construction perform no HTTP requests; verify the local runtime and normal CLI paths retain their transport-free boundary.

## 3. Disposable external conformance experiment

- [x] 3.1 Design and implement a narrowly scoped, explicitly confirmed Playwright experiment harness that creates and uses only a fresh disposable online character; verify it refuses a managed-character input or any run without confirmation.
- [x] 3.2 Limit browser-harness network access to the observed official endpoints and retain the passkey only in its ephemeral browser profile; verify errors, logs, and persisted evidence are credential-free.
- [x] 3.3 Execute paired browser and Gyrognome scenarios for initial load, pause, restart, delayed callbacks, task completion, level-up, act completion, manual bragging, and motto change; verify their recorded report traces match before a browser-originated external submission.
- [x] 3.4 Poll within documented bounds for normal leaderboard placement and absence from the cheater population after each required scenario; verify an unavailable, inconclusive, or cheater-classified result fails the gate. Do not port native `newguy` character generation as part of this task.

## 4. Evidence and delivery gate

- [ ] 4.1 Document the disposable-character procedure, confirmation requirements, evidence format, polling bounds, and failure handling; verify the instructions never ask an operator to use a real character save or publish a passkey.
- [ ] 4.2 Record a credential-free conformance evidence summary for all required scenarios; verify it identifies the observed browser revision and reports normal-versus-cheater classification results without bearer data.
- [ ] 4.3 Run formatting, warning-denied linting, report-trace tests, fixture-safety tests, runtime-boundary tests, and the complete test suite; verify all automated checks pass and the external evidence gate is explicitly passing before proposing general leaderboard transport.
