## 1. Project foundation

- [x] 1.1 Create the Rust 2024 binary crate with pure state, RNG, save, protocol, and CLI module boundaries; verify `cargo check` succeeds without network-client dependencies.
- [x] 1.2 Add typed error handling, JSON/Base64 serialization, and command-line parsing dependencies; verify `cargo test` runs successfully on the new crate.
- [x] 1.3 Add test-fixture and diagnostics conventions that reject committed real passkeys, player save exports, full signed request URLs, and browser profiles; verify the fixture-safety test detects representative prohibited values.

## 2. Browser save compatibility

- [x] 2.1 Implement Base64 JSON `.pqw` import with strict validation of required canonical character state; verify valid sanitized browser fixtures import and malformed Base64, JSON, and required fields produce explicit errors.
- [x] 2.2 Retain the raw imported document with a typed canonical projection and implement unmodified export; verify an import/export/re-import fixture retains canonical state and unrecognized JSON fields.
- [x] 2.3 Implement a read-only `inspect` command that summarizes canonical character state and redacts online passkeys; verify its output identifies online realm without exposing the synthetic fixture passkey.

## 3. Deterministic compatibility primitives

- [x] 3.1 Port Alea state initialization, restoration, and bounded-random behavior with explicit browser-compatible numeric semantics; verify sanitized fixture sequences and restored-state continuations exactly match expected values.
- [x] 3.2 Implement the browser-compatible signed 32-bit LFSR primitive; verify known synthetic input, salt, and validator-output fixtures.
- [x] 3.3 Implement exact request parameter ordering, percent encoding, and URL normalization used by the browser client; verify generated synthetic creation, progress-report, guild, and motto request data against fixtures.

## 4. Reference observations and conformance coverage

- [x] 4.1 Use the configured Playwright MCP with a disposable online browser character to document normal creation and milestone-report behavior without modifying personal saves; verify the observation record is reviewed and contains no reusable credentials.
- [x] 4.2 Convert browser observations into synthetic, credential-free save, PRNG, and request fixtures with documented source revision; verify fixture tests do not require browser access or a live leaderboard.
- [x] 4.3 Add end-to-end offline conformance tests for save inspection and report construction, and confirm no crate dependency or execution path performs HTTP transport; verify `cargo test` passes and dependency inspection contains no HTTP client.

## 5. Offline-only delivery

- [x] 5.1 Document supported browser-save interchange, deterministic compatibility guarantees, fixture provenance, and the explicit prohibition on leaderboard submission; verify the documented CLI examples run against synthetic fixtures.
- [x] 5.2 Run formatting, linting, and the complete test suite; verify the project reports no warnings or failures before the change is submitted for review.
