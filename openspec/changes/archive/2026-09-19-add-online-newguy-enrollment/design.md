# Design

## Context

The New Guy wizard currently returns an offline canonical character to the CLI,
which immediately registers it. `protocol::create_request` already produces
the unsigned browser-compatible create URL, and foreground reporting already
uses a narrow HTTPS boundary and validates bundled enrollment evidence. The
store writes canonical state and a private original document atomically, but
cannot atomically include remote creation or reporting. See proposal.md and
the enrollment specs for the behavior contract.

## Goals / Non-Goals

**Goals:**

- Keep an online draft in memory until the create request and initial `s`
  report both complete successfully.
- Restrict production enrollment to the official Alpaquil HTTPS endpoint and
  keep credentials in the store's private original-document column.
- Reuse browser-compatible request construction and the current terminal
  wizard rather than add a second character editor.

**Non-Goals:**

- Automatic retries, resumable enrollment, name reservation recovery,
  enrollment from scripts, endpoint selection, or background transport.
- Claiming whether an unusable create response reserved a name.
- Persisting response bodies, credentials in fixtures or diagnostics, or
  changing runtime progression and scheduled reporting.

## Decisions

### Select enrollment as a creation-wizard row

When `new-guy` is invoked without explicit traits, the shared editable draft
will add a Mode row alongside Name, Race, Class, and Stats. It will begin as
Offline and use the same focused-row interaction pattern as the existing
selectable rows. Switching Mode to Online will preserve the provisional draft
and make Sold!'s enrollment semantics visible; switching it back retains the
same draft and restores offline-only Sold! behavior. Explicit `--name`,
`--race`, and `--class` inputs remain the existing offline-only scripted path;
they will not select or perform online enrollment.

A CLI flag was rejected because the mode belongs to the character-creation
experience, consistent with the browser client. A noninteractive online
command was rejected because it would bypass the interactive Sold!
confirmation.

### Model Sold! as a one-shot enrollment service

The wizard will return the validated final draft to a dedicated foreground
enrollment service rather than a completed character. The service will first
validate the bundled passing evidence, issue exactly one official create
request, parse only the minimum successful enrollment fields needed to attach
online metadata and a private credential, construct the browser-equivalent
initial `s` report, and require a successful delivery result before calling
`Store::register`.

The existing generic manual-report path cannot be reused directly because it
requires an already registered character. Registering before the initial
report was rejected because an incomplete activation would become a usable
local online identity contrary to the agreed fail-closed policy.

### Treat only a known duplicate-name response as editable

The response parser will classify an endpoint-confirmed duplicate-name
rejection separately from a successful usable enrollment result and all other
outcomes. Only the duplicate case returns control to the same draft. Every
other response, parse, credential, create-delivery, or initial-report failure
ends the activation without retry or persistence.

Retrying a request with an uncertain server outcome was rejected because the
server may already have reserved the name. Persisting a recovery record was
rejected by the confirmed fail-closed policy.

### Assemble a private source document only after successful enrollment

On the successful path, the service will use the finalized New Guy state,
official online realm/host, and validated credential to build the minimal
private document required by existing storage and future foreground reporting.
The public canonical state retains only realm and host. Construction and error
types will retain no complete signed URL or response body.

Changing the SQLite schema was rejected because the private original document
already provides the credential retention boundary used by imported online
characters.

## Risks / Trade-offs

- [The server accepts create but a response or initial report cannot be used]
  -> End the activation without retry or local registration, and state only
  that enrollment is incomplete.
- [The browser/server response contract changes] -> Validate the bundled
  evidence before transport and add response-parser tests using
  credential-free synthetic payloads.
- [A response or HTTP error leaks a credential] -> Map failures to fixed safe
  outcomes and discard raw response bodies after private parsing.
- [Terminal failure occurs during an activation] -> Restore the terminal and
  avoid registration unless the full remote sequence has already succeeded.

## Migration Plan

1. Add an in-wizard Mode row for Offline/Online selection while retaining
   explicit trait inputs as the offline-only scripted path.
2. Add the constrained enrollment transport, success/rejection classifier, and
   private online-state assembly behind the evidence gate.
3. Wire the one-shot Sold! activation so registration follows a successfully
   delivered initial report only.
4. Add unit, CLI integration, fixture-safety, and runtime-boundary tests.
5. Roll back by removing the online mode and enrollment service; existing
   locally registered characters and the offline New Guy flow remain intact.
