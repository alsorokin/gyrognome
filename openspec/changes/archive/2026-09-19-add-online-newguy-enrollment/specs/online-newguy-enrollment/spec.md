# Spec Delta

## Purpose

Enable an explicitly selected New Guy wizard flow to create a new online
character through the official Progress Quest endpoint without exposing its
credential or persisting an incomplete local identity.

## ADDED Requirements

### Requirement: Explicit online enrollment activation

The interactive New Guy flow SHALL provide a navigable Mode row alongside its
editable Name, Race, Class, and Stats rows. The Mode row SHALL offer Offline
and Online. When the user selects Online, the draft SHALL identify that Sold!
will create an online character. Changing the Mode SHALL retain the
provisional name, race, class, and stats. The system SHALL perform no network
request before the user selects Sold!.

#### Scenario: Opening the online enrollment wizard

- **WHEN** a user selects Online in the Mode row
- **THEN** the system opens an editable online-enrollment draft without
  registering a local character or making an HTTP request

#### Scenario: Switching to online enrollment

- **WHEN** a user changes the Mode row from Offline to Online
- **THEN** the system retains the provisional name, race, class, and stats
  while changing only the Sold! activation behavior

#### Scenario: Cancelling online enrollment

- **WHEN** a user cancels the online enrollment wizard before Sold!
- **THEN** the system restores the terminal and creates neither an online
  identity nor a local managed character

### Requirement: Browser-compatible successful enrollment

The system SHALL validate the bundled passing enrollment-conformance evidence
before sending an enrollment request. Upon Sold!, it SHALL validate the draft
name, send one browser-compatible `cmd=create` request to the official
Alpaquil endpoint, privately validate the successful enrollment result and its
credential, and send one browser-compatible initial `s` report using the final
draft state. It SHALL register one managed character only after both remote
operations succeed. The persisted character SHALL contain the official online
realm and endpoint and retain the credential only in private managed storage.

#### Scenario: Enrolling and registering a new online character

- **WHEN** a user selects Sold! for a valid online enrollment draft and the
  creation request and initial `s` report both succeed
- **THEN** the system registers one online managed character and reports only
  its credential-safe managed identity

#### Scenario: Rejecting unavailable enrollment evidence

- **WHEN** the bundled enrollment-conformance evidence is unavailable,
  malformed, sensitive, or non-passing
- **THEN** the system sends no creation or report request and registers no
  managed character

### Requirement: Server-authoritative name rejection

The system SHALL treat the creation endpoint as the sole authority for name
availability and SHALL not issue a name-availability preflight. When the
endpoint rejects the draft name, the system SHALL send no initial report,
shall not register a managed character, and SHALL retain the editable draft
with a credential-safe rejection message.

#### Scenario: Correcting a rejected name

- **WHEN** the creation endpoint rejects a Sold! draft because its name is
  unavailable
- **THEN** the wizard remains open with the draft available for editing and
  without a local registration or initial report

### Requirement: Fail-closed incomplete enrollment

The system SHALL classify an unusable creation response, invalid credential,
creation delivery failure, rejected initial report, or initial-report delivery
failure as incomplete enrollment. For an incomplete enrollment it SHALL not
retry either remote operation, register a managed character, expose a
credential, response body, or signed URL, or claim that the server did or did
not reserve the name.

#### Scenario: Losing the creation response

- **WHEN** the creation request cannot yield a usable enrollment result
- **THEN** the system ends the enrollment activation with a credential-safe
  incomplete-enrollment error and no local registration

#### Scenario: Failing the initial report

- **WHEN** creation yields a usable online credential but the required initial
  `s` report is not delivered successfully
- **THEN** the system ends the enrollment activation without retrying or
  registering a managed character

### Requirement: Foreground-only enrollment boundary

The system SHALL perform enrollment only from the explicitly selected,
foreground New Guy wizard. Character import, offline New Guy creation,
runtime progression, lifecycle operations, dashboard refresh, administration,
and manual reporting SHALL not create an online identity or invoke enrollment
transport. User-visible output and errors SHALL not expose a passkey, raw
creation response, retained private document, or complete signed request URL.

#### Scenario: Running unrelated character operations

- **WHEN** a user imports, creates an offline character, advances a runtime,
  refreshes a dashboard, administers a character, or submits a manual report
- **THEN** the operation does not invoke online character enrollment
