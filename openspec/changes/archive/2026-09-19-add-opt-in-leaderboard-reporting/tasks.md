## 1. Eligibility and conformance gate

- [x] 1.1 Add a Rust validator for the bundled credential-free enrollment-conformance evidence and safe error types; verify valid evidence passes while missing, sensitive, non-live, or non-passing evidence fails without exposing its contents.
- [x] 1.2 Add private store access that resolves an inactive imported online character's current canonical state and passkey without expanding public inspection or diagnostic models; verify offline, malformed, missing-credential, unknown, and runtime-owned targets are rejected with safe errors.

## 2. Explicit foreground reporting

- [x] 2.1 Add a synchronous HTTPS reporting service that constructs exactly one browser-compatible `t=b` request for an eligible target and restricts delivery to the official leaderboard endpoint; verify local mock tests cover successful delivery, non-success status, and connection failure without retaining response bodies or signed URLs.
- [x] 2.2 Add the dedicated CLI reporting command with credential-safe identity display and per-submission interactive confirmation; verify declined confirmation sends no request, successful delivery reports only a delivered outcome, and all errors remain credential-free.

## 3. Boundary coverage and delivery

- [x] 3.1 Extend runtime-boundary, CLI, and fixture-safety tests to prove registration, inspection, dashboard refresh, workers, lifecycle operations, and character administration remain transport-free, while only the confirmed reporting command can invoke the reporting service.
- [x] 3.2 Document the browser-imported-character eligibility rules, confirmation flow, safe delivery outcomes, and non-goals; verify documentation never instructs users to provide a passkey, raw save content, signed URL, or native-enrollment character name.
- [x] 3.3 Run formatting, warning-denied linting, targeted transport and CLI tests, fixture-safety and runtime-boundary suites, and the full Rust and Node test suites; verify normal CLI paths remain transport-free and no committed artifact contains a credential.
