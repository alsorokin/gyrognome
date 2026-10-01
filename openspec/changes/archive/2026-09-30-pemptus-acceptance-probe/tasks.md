# Tasks

## 1. Native diagnostic and scope validation

- [x] 1.1 Add the feature-gated `pemptus-acceptance-probe` binary and narrow backing module using existing dependencies; verify the binary is absent from default builds and its help lists validation-only and explicit live confirmations.
- [x] 1.2 Implement read-only canonical import, disposable level-1 handoff, exact Pemptus realm/endpoint, positive passkey, empty credentials, and ASCII validation; verify synthetic tests reject each invalid precondition before networking and validation-only mode makes zero requests.
- [x] 1.3 Reuse the existing Rust manual-report constructor without advancing state; verify synthetic vectors cover field order, revision 8, absent specialty for empty spells, passkey validator, and retained motto.

## 2. Bounded transport and observations

- [x] 2.1 Implement experimental HTTPS delivery to the fixed Pemptus endpoint with no Authorization header, redirects, or retries; verify injected transport tests observe at most one mutation and cover rejection, timeout, disconnect, redirect, and oversized bodies without leaking secrets.
- [x] 2.2 Implement bounded public baseline and post-report observations using exact row identity and fame/infamy headings, reusing or extracting compatible existing helpers; verify synthetic pages cover matching rows, similarly named characters, name text outside rows, malformed pages, missing rows, and cheater classification.
- [x] 2.3 Compare report-visible state and expose categorized delivery, classification, agreement, and observable change; verify an unchanged matching baseline or explicit rejection never becomes an acceptance claim.
- [x] 2.4 Enforce the 60-second post-submission deadline across requests and sleeps; verify controllable-clock/transport tests establish the actual bound and no automatic second mutation occurs after failure.

## 3. Documentation and integration

- [x] 3.1 Document the probe command, disposable handoff, absence of persistent private artifacts, renewed authorization after an ambiguous attempt, and manual-only acceptance limits in `docs/classic-desktop-compatibility.md`; verify examples match binary help and do not include real identities or credentials.
- [x] 3.2 Run focused probe, desktop protocol, existing live-runner, and production gate regression tests, plus formatting and feature-gated build checks; verify Pemptus production eligibility remains blocked and Spoltog behavior is unchanged.

## 4. Approved one-shot experiment

- [x] 4.1 Validate the approved local Pemptus `.pq` input without network activity and confirm that the disposable-character/stopped-client authorization remains current; verify no source-save changes and no managed registration or private experiment files are created.
- [x] 4.2 Execute exactly one approved Pemptus manual-brag attempt, with public observation bounded to 60 seconds and no retries, progression, or motto/guild mutation; report only safe delivery/classification/state-agreement categories and any acceptance uncertainty.
- [x] 4.3 Verify source-save hashes and production eligibility remain unchanged and record only a credential-free outcome in the change's task record; if delivery was attempted, never repeat it without renewed operator authorization.

## Credential-free execution outcome

On 2026-09-30, after renewed stopped-client and execution confirmation, the
probe attempted exactly one native passkey-only Pemptus manual report. Delivery
returned HTTP 200 with an empty response. Baseline and post-report
classification were normal; the observed report-visible state matched the
submitted state but was unchanged from the baseline. Post-report observation
completed in approximately 0.66 seconds with no observation error.

Mutation acceptance was not independently established by the unchanged public
row. This result is diagnostic only and does not enable production eligibility
or constitute full Pemptus conformance evidence. No retry, progression,
motto/guild mutation, managed registration, or private experiment artifact was
performed. Both source saves remained byte-for-byte unchanged, and all five
production Pemptus operations remained gated.

The feature-gated build, ten probe tests, and 111 desktop regression tests
passed. Formatting and change validation passed. The new probe produced no
Clippy diagnostics; strict repository-wide warning denial encountered eight
pre-existing diagnostics in unrelated code, which was left unchanged.
