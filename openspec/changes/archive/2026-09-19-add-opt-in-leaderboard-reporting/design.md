## Context

The compatibility core can construct signed browser-compatible reports, while
the local store retains the original imported browser document privately.
Existing runtime-boundary tests prohibit all transport, and the completed
enrollment-conformance fixture establishes the browser's safe enrollment and
initial-report observations. See proposal.md for motivation and the delta specs
for behavioral requirements.

## Goals / Non-Goals

**Goals:**

- Add one explicit, synchronous reporting operation for an inactive
  browser-imported managed character.
- Use the existing browser-compatible request construction and private
  credential storage without expanding the safe public character model.
- Fail closed before networking on ineligible characters or invalid bundled
  conformance evidence, and preserve state on every delivery outcome.

**Non-Goals:**

- Native character enrollment, scheduled reports, automatic retries, report
  queues, leaderboard classification polling, or a UI workflow.
- Exposing or exporting a passkey, signed URL, raw original document, response
  body, or transport diagnostics.
- Changing worker timing, service lifecycle, local progression, or dashboard
  refresh behavior.

## Decisions

### Submit only an explicit manual-brag event

The new CLI operation will submit one `t=b` report built from the selected
character's current persisted canonical state. This aligns submission with an
intentional operator action and avoids silently deciding how simulation events
should be transported. Supporting arbitrary triggers or automatic lifecycle
reports is deferred because each needs distinct ordering and retry semantics.

### Keep credential access inside a foreground reporting service

A reporting module will receive a store, resolve an inactive character, load
its retained original document internally, extract and validate the online
credential, construct the request, and send it to the fixed official HTTPS
endpoint. CLI presentation receives only a safe identity and outcome enum.
Passing signed URLs or passkeys through command, logging, or error types is
rejected because those values are bearer credentials.

### Gate delivery on bundled conformance evidence

The credential-free enrollment evidence fixture will be embedded or otherwise
read as a package-controlled artifact and validated through a Rust equivalent
of its schema before transport setup. A missing, malformed, sensitive, or
non-passing artifact blocks submission. Runtime user data cannot override this
gate; a source or fixture revision changes only through code review.

### Treat a successful HTTP status as delivery, not enrollment or classification

The transport will expose a narrow result: delivered, endpoint-rejected, or
delivery-failed. It will never retain or surface the response body, and it will
not poll the leaderboard or infer classification. This avoids conflating
network reachability with external server state.

### Preserve the default transport-free boundary

The reporting service is called only by the dedicated CLI command after a
fresh explicit confirmation. Workers, services, dashboard, registration,
inspection, administration, and all error paths must remain unable to invoke
it. This is preferred to a shared background client because it preserves the
current offline default and keeps consent local to one command.

## Risks / Trade-offs

- [A request succeeds remotely but the process exits before presenting it] →
  avoid automatic retries and report only the observed local delivery result.
- [A retained imported document is malformed or lacks online credentials] →
  validate it before constructing the request and return a safe eligibility
  error.
- [A reporting attempt races with a local worker] → acquire or inspect the
  character ownership guard before credential access and refuse when owned.
- [A transport error includes sensitive request details] → map library errors
  to fixed safe categories and discard response bodies.
- [The official endpoint or browser protocol changes] → conformance evidence
  and browser-compatible request tests fail closed before delivery.

## Migration Plan

1. Add the foreground reporting command, eligibility checks, and safe
   confirmation/output types behind the existing managed-character store.
2. Add a minimal synchronous HTTPS transport restricted to the official
   endpoint and test it through a local mock boundary without recording signed
   requests.
3. Add conformance-evidence validation and boundary tests proving all ordinary
   runtime paths remain transport-free.
4. Roll back by removing the reporting command and transport module; retained
   browser documents and canonical state remain compatible and local-only.
