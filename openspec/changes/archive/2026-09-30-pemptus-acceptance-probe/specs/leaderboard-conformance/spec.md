# Spec Delta

## ADDED Requirements

### Requirement: Bounded Pemptus manual-report diagnostic

The project SHALL provide a development-only native diagnostic for an explicitly
approved newly created disposable Pemptus character established through the
official desktop client. Execution SHALL require confirmation that the official
client is stopped and that one live manual-brag request is authorized.
Validation-only execution SHALL contact no server.

The diagnostic SHALL accept only a supported desktop-6.4.4 canonical import,
Pemptus realm, exact saved endpoint `http://progressquest.com/pemptus.php?`,
positive passkey, empty account/password credentials, and supported ASCII
request data. It SHALL construct the manual report from the imported state
without progression and send it only to `https://progressquest.com/pemptus.php`,
without an Authorization header, redirects, HTTP downgrades, or mutation
retries. It SHALL preserve source files and SHALL NOT register a managed
character, change motto/guild, or save credentials or signed requests.

#### Scenario: Approved diagnostic execution

- **WHEN** an approved disposable import satisfies all preconditions
- **THEN** at most one native manual-brag request is attempted, and no other
  mutation or simulation advancement occurs

#### Scenario: Invalid or unapproved scope

- **WHEN** confirmation is missing or the realm, endpoint, credentials, encoding,
  or supported import path does not match the diagnostic contract
- **THEN** execution fails with a credential-safe explanation before any network
  activity

#### Scenario: Ambiguous mutation delivery

- **WHEN** the sole mutation attempt times out, disconnects, redirects, or has an
  indeterminate response
- **THEN** its outcome is categorized safely, the mutation is not retried, and a
  later mutation requires new explicit operator authorization

### Requirement: Honest and credential-safe diagnostic observations

The diagnostic SHALL make bounded credential-free public observations of the
exact character before submission and for at most 60 seconds afterward.
Each post-submission observation request and wait SHALL be bounded by the
remaining observation deadline. Responses SHALL have explicit size limits.

Output SHALL distinguish transport delivery, explicit rejection, normal,
cheater, or unknown public classification, and whether observable public state
matches the submitted report. A preexisting unchanged normal row SHALL NOT be
presented as proof that the mutation was accepted. Public-page ambiguity,
malformed data, size-limit failure, or observation timeout SHALL remain
inconclusive rather than being converted into acceptance.

Output SHALL exclude passkeys, credentials, signed URLs, source-save contents,
and raw or normalized response bodies. Diagnostic results SHALL NOT enable
production operations or substitute for full scoped classic-realm live evidence.

#### Scenario: Delivered report with a matching normal row

- **WHEN** a report is delivered and the exact character is observed in the
  normal population with matching report-visible state
- **THEN** delivery, normal classification, and state agreement are reported
  separately, without claiming full Pemptus conformance

#### Scenario: Public state was already identical

- **WHEN** the normal public row matches both the baseline and submitted state
- **THEN** the diagnostic states that mutation acceptance was not independently
  established by the unchanged observation

#### Scenario: Cheater or missing observation

- **WHEN** the exact character is classified as a cheater or cannot be classified
  within the observation bound
- **THEN** execution reports cheater or inconclusive status respectively and
  leaves production reporting eligibility unchanged
