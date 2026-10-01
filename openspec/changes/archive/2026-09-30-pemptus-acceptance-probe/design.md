# Design

## Context

See proposal.md for motivation. The existing Rust importer already accepts the
observed Pemptus saves without adaptations, and desktop_protocol::report
supports empty account/password credentials. Published desktop 6.4.4 sources
use shared simulation and request construction across realms and omit account
authentication when credentials are absent.

The existing desktop-live-conformance runner hardcodes Spoltog, always emits
Basic authentication, writes private progression checkpoints, and performs
multiple mutations. It is not suitable for this one-request experiment.
Production desktop transport likewise assumes a verified account/password
contract. Those gates and bundled evidence must remain unchanged.

## Goals / Non-Goals

**Goals:** Exercise existing native import and protocol code through a minimal
development-only network adapter, with observable and testable attempt bounds.

**Non-Goals:** Rework the production realm registry, promote diagnostic results
to production evidence, or generalize the full progression runner.

## Decisions

### Separate feature-gated diagnostic entry point

Add a `pemptus-acceptance-probe` binary under the existing
`desktop-live-conformance` feature, backed by a narrowly scoped Rust module.
Accept a source-save path rather than a managed character id. Provide explicit
disposable, stopped-client, and live-submission confirmations and a
validation-only mode. Validate level-1/new-character handoff and the canonical
unadapted import path before networking.

Reusing the multi-operation runner would introduce checkpointing, cleanup
mutations, and account assumptions outside the approved scope. A separate
entry point also preserves its Spoltog behavior.

### Reuse canonical request construction; isolate experimental transport

Use import_supported_file and desktop_protocol::report with the manual
operation, retained passkey, existing motto, and empty account authentication.
Build the signed query only in memory. Pin the destination to the exact
Pemptus HTTPS endpoint; never derive trust from the save.

Use the project's existing HTTPS-only HTTP library, no Authorization header,
no redirects, no automatic retries, and explicit response size/time bounds.
Keep the experimental path separate from production eligibility and endpoint
mapping. Share existing bounded observation/parsing helpers where their
contracts fit; extract small helpers only when needed rather than duplicating
the whole live runner.

### Separate delivery from observable acceptance

Capture a bounded public baseline before sending. If baseline fetching fails,
abort before mutation with an explicit safe error. After the single mutation
attempt, inspect exact character table rows under their relevant fame/infamy
heading, not arbitrary name substrings elsewhere in HTML.

Compare report-visible columns to the imported snapshot using existing
desktop selection rules. Do not compare unsent internal progress bars or
inventory details. Poll for at most 60 seconds with each request timeout and
sleep clipped to the remaining monotonic deadline.

Expose categorized delivery, classification, state agreement, and observable
change, never raw response text. HTTP success with an unchanged normal row is
delivery plus corroborating classification, not independent proof of mutation
acceptance. An explicit response rejection must not be overridden by a
preexisting normal row.

### No persistent experiment state

The diagnostic writes no private checkpoints, managed database records, or
response artifacts. Inputs remain read-only; diagnostics retain secrets in
memory only. One invocation performs at most one mutation. A failed or
inconclusive attempt ends the approved mutation scope; no automatic rerun is
permitted, and a subsequent invocation requires renewed operator approval.

## Risks / Trade-offs

- [An already matching leaderboard row cannot prove acceptance] -> Report this
  limitation explicitly; do not add unapproved motto markers or progression.
- [Ambiguous network delivery could already have mutated the server] -> Never
  retry and distinguish attempted delivery from definite response receipt.
- [Official client resumes concurrently] -> Require stopped-client confirmation;
  the probe cannot remotely enforce operator-controlled client shutdown.
- [Public HTML changes or contains ambiguous names] -> Match exact row identity,
  fail inconclusively, and cover malformed/misleading pages with synthetic tests.
- [Probe success is mistaken for general support] -> Preserve all production
  gates and label output as a manual-only diagnostic.

## Migration Plan

No database or save migration is required. Build the development binary only
with its feature enabled. Run synthetic checks before the single approved
live execution. Removing the new entry point restores the prior development
surface without altering production evidence or character state.
