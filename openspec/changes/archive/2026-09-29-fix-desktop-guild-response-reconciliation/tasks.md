## 1. Shared Guild Fingerprint Contract

- [x] 1.1 Extract the live-conformance response normalization and SHA-256 logic into one versioned shared implementation, preserving the existing ordered replacement behavior and `<redacted>` marker; verify fixed synthetic compatibility vectors, exact algorithm-preservation tests, and unchanged bundled evidence validity without reconstructing deleted live response bodies.
- [x] 1.2 Replace the duplicate production guild normalizer with the shared implementation and pass character name, account, password, Basic authorization value, decimal passkey, prior guild, and submitted guild in the evidence-defined order; verify production classification recognizes the bundled fingerprints without exposing dynamic values.
- [x] 1.3 Update live-conformance generation to call the shared implementation and record or validate its normalization version as appropriate for the existing evidence schema; verify the current sanitized fixture remains valid without a new live experiment.
- [x] 1.4 Add cross-path tests that fingerprint credential-free accepted join/change, invalid rejection, and accepted leave vectors through the evidence path and classify them through production, including identity/authentication text and overlapping dynamic values; verify the tests would fail if either replacement marker, value set, order, or encoding diverges.

## 2. Indeterminate Response Reconciliation

- [x] 2.1 Refine the bounded verified-realm public-profile fetch so it performs no authenticated request, follows no unsafe redirect, and enforces the existing response-size limit; verify focused transport tests cover allowed, oversized, redirect, and delivery-failure outcomes.
- [x] 2.2 Refine the public leaderboard parser to require the exact character row and guild column, decode only supported entities, and distinguish an explicit empty guild from absent, malformed, or ambiguous data; verify parser tests cover omitted closing tags, duplicate or missing rows, malformed cells, canonical capitalization, and unrelated characters.
- [x] 2.3 Integrate reconciliation only after an indeterminate desktop guild classification, permitting at most one credential-free public read and no repeated mutation; verify reporting tests assert one mutation, zero fallback reads for recognized responses, one fallback read for indeterminate responses, and fail-closed behavior for absent, ambiguous, malformed, delayed, or mismatched observations.
- [x] 2.4 Persist the server-observed canonical ASCII capitalization for verified joins/changes and clear the guild only for a verified explicit leave, while preserving prior state for every unverifiable outcome; verify focused state tests cover accepted, rejected, reconciled, and preserved outcomes.

## 3. Specification Workflow

- [x] 3.1 Keep the main `online-character-profile` and `opt-in-leaderboard-reporting` specs at their pre-change requirements while retaining the new behavior in this change's delta specs; verify `git diff` shows these behavioral spec edits only under `openspec/changes/fix-desktop-guild-response-reconciliation/specs`.
- [x] 3.2 Verify the implementation provides every scenario in the leaderboard-conformance, online-character-profile, and opt-in-leaderboard-reporting delta specs through the shared classifier, bounded transport, exact-row parser, and one-read reconciliation tests.

## 4. Validation and Delivery

- [x] 4.1 Run Rust formatting and the focused desktop profile, live-conformance, evidence, transport, reporting, and public-profile parser tests in Ubuntu WSL; verify all commands pass without modifying bundled credentials or creating private artifacts.
- [x] 4.2 Synchronize the Windows checkout to the disposable ext4 validation mirror and run the full default Rust test suite, all-feature Rust test suite, and Node test suite; verify every suite passes from the synchronized mirror.
- [x] 4.3 Run strict OpenSpec validation for `fix-desktop-guild-response-reconciliation` and verify the proposal, three delta specs, design, and task checklist are valid and complete.
- [x] 4.4 Install the validated binary to the WSL user cargo bin and verify the installed CLI reports the expected version/help behavior without issuing any leaderboard mutation.
