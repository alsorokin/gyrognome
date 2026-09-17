## Context

The existing conformance harness creates disposable browser characters to
compare reporting behavior, but its evidence intentionally does not describe
the `Sold!` enrollment response or a duplicate-name attempt. The compatibility
core can construct a `cmd=create` request but has no HTTP client, response
contract, or normal-user enrollment path. See proposal.md for the motivation.

## Goals / Non-Goals

**Goals:**

- Produce repeatable, credential-free evidence for the browser's successful,
  rejected, and interrupted enrollment behavior.
- Establish the browser-observed ordering of `cmd=create` and the initial `s`
  report that a later transport feature must preserve.
- Preserve the existing disposable-character confirmations, request allowlist,
  and fixture-safety model.

**Non-Goals:**

- Add a production HTTP dependency, CLI command, managed-character migration,
  or native online enrollment UI.
- Record creation response bodies, passkeys, raw saves, or complete signed
  request URLs.
- Decide how a future production client resolves an enrollment request whose
  server-side outcome is unknowable.

## Decisions

### Extend the disposable browser harness rather than emulate enrollment

The existing Playwright harness will drive the official browser's New Guy
experience and observe its network activity. It will record a redacted
descriptor for each relevant request—operation, request order, endpoint,
method, and unsigned field names—plus browser-visible success or failure
outcomes.

The compatibility core alone cannot establish server responses, and a native
emulation would test Gyrognome's assumptions rather than browser behavior.

### Prove duplicate-name behavior through a prior disposable reservation

One browser-created disposable character will reserve a generated name. A
separate disposable browser context will attempt the same name and observe the
browser's rejection. The experiment will not use a user-provided or
pre-existing character name, nor perform an availability preflight.

An availability lookup was rejected because it cannot reserve a name and
therefore has a time-of-check/time-of-use race with actual enrollment.

### Treat interrupted enrollment as unknowable

The harness will block delivery of a usable creation response and record the
browser's visible error and whether the browser issues another creation
request. Evidence will label this outcome unconfirmed; it must not derive
server acceptance, credential possession, or a valid online identity.

Forcing a server-side accepted request while discarding its response was
rejected for this change because it could orphan a live disposable character
and still cannot establish a general recovery protocol.

### Fail closed on evidence and live requests

The existing confirmation requirements remain mandatory. Dry runs will
intercept creation and report requests; live requests require the existing
separate live-submission confirmation. Evidence serialization and fixture
validation will reject credentials, response bodies, profile paths, raw saves,
and signed URLs before output or commit.

Making conformance requests implicit would violate the project boundary and
could create a live online identity without the operator's informed consent.

## Risks / Trade-offs

- [The official client or server response changes] → Pin observed client
  revision and source hash, and fail the evidence gate when observations no
  longer match.
- [A duplicate-name test creates or reports another character] → Use a
  browser-created disposable reservation and intercept all requests from the
  duplicate-attempt context after the creation outcome is observed.
- [Response diagnostics contain a passkey] → Never serialize response bodies;
  retain only a derived success/rejection/unconfirmed outcome.
- [A blocked response does not model every network failure] → Record the
  specific interception mode and limit the claim to browser-visible behavior,
  leaving production recovery policy for the later enrollment change.

## Migration Plan

1. Add redacted enrollment observation and evidence validation to the existing
   disposable harness.
2. Run confirmed disposable dry runs and the explicitly confirmed live
   experiment where required for the evidence gate.
3. Commit only passing credential-free fixtures and documentation.
4. Roll back by removing the enrollment-observation path and its artifacts;
   the normal CLI and managed-character database remain unchanged.
