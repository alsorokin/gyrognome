## Purpose

Establish credential-safe, browser-derived evidence for the online character
enrollment and first-report sequence before normal-user transport is enabled.

## ADDED Requirements

### Requirement: Disposable browser enrollment conformance

The project SHALL provide an explicitly confirmed conformance procedure that
uses the official browser client to create newly generated, disposable online
characters. The procedure SHALL capture credential-free evidence of the
browser's creation request shape, successful enrollment outcome, and the
ordered relationship between successful creation and the initial `s` report.
It SHALL not expose, persist, or print a passkey, response body, browser
profile, raw browser save, or complete signed request URL.

#### Scenario: Recording successful enrollment

- **WHEN** an operator explicitly confirms a disposable enrollment experiment
- **THEN** the procedure records credential-free creation and initial-report
  observations from a browser-created disposable character

#### Scenario: Rejecting unconfirmed enrollment

- **WHEN** an operator does not supply the required disposable-experiment
  confirmation
- **THEN** the procedure performs no online character-creation request

### Requirement: Server-authoritative duplicate-name evidence

The conformance procedure SHALL demonstrate the browser's response to an
attempt to create a second disposable character with a name already reserved
by a preceding disposable enrollment. The evidence SHALL distinguish rejected
name reservation from successful enrollment and record only the browser-visible
outcome and credential-free request metadata. It SHALL not query or rely on a
separate name-availability endpoint.

#### Scenario: Rejecting a duplicate disposable name

- **WHEN** the browser attempts enrollment using a name reserved by a prior
  disposable browser enrollment
- **THEN** the procedure records the browser's rejection outcome without
  treating the name as an enrolled local character

### Requirement: Ambiguous enrollment failure evidence

The conformance procedure SHALL capture the browser-visible outcome when an
enrollment request cannot yield a usable response. It SHALL record whether the
browser retries the creation request and SHALL classify the result as
unconfirmed rather than successful enrollment. The procedure SHALL not infer
whether the server reserved the name from an interrupted or unavailable
response.

#### Scenario: Interrupting enrollment before a usable response

- **WHEN** the conformance procedure prevents the browser from receiving a
  usable enrollment response
- **THEN** it records the browser-visible failure and retry behavior, and
  produces no successful-enrollment evidence

### Requirement: Enrollment evidence gate

The project SHALL reject enrollment conformance evidence that omits any
successful-creation, duplicate-name, initial-report-order, or interrupted-
enrollment observation, or that contains credential-bearing data. A later
native enrollment feature SHALL remain unsupported unless the recorded
evidence passes this gate.

#### Scenario: Detecting incomplete evidence

- **WHEN** a committed enrollment evidence artifact lacks a required
  observation or contains prohibited credential-bearing data
- **THEN** validation fails and does not mark online enrollment conformant
