# Proposal

## Why

Gyrognome can create local-only New Guy characters and has credential-free
browser evidence for online enrollment, but it cannot create a managed online
character. Users therefore cannot begin a new online character without first
using the browser and importing its save.

## What Changes

- Add a foreground-only native online New Guy enrollment flow that uses the
  existing creation wizard's displayed draft.
- Send the server-authoritative name-creation request and then the
  browser-equivalent initial `s` report as one explicit Sold! activation.
- Preserve an editable draft after a duplicate-name rejection.
- Fail closed after an ambiguous creation response or failed initial report:
  do not retry, persist, or claim an online managed character.
- Keep passkeys, response bodies, signed URLs, automatic reporting, and
  transport access outside the enrollment path from safe output and unrelated
  commands.

## Capabilities

### New Capabilities

- `online-newguy-enrollment`: Create and locally register a new online
  character only after browser-compatible server enrollment and initial
  reporting complete successfully.

### Modified Capabilities

- `offline-newguy`: Preserve the existing offline-only behavior as an explicit
  creation mode while allowing the interactive New Guy experience to select
  the distinct online enrollment flow.

## Impact

- Affected systems: New Guy terminal wizard and CLI, protocol construction,
  foreground HTTPS transport, canonical online metadata/original-document
  handling, managed-character registration, and integration tests.
- The existing bundled enrollment-conformance evidence becomes a mandatory
  precondition for native enrollment.
- No background transport, scheduler reporting, database migration, or new
  external dependency is planned.
