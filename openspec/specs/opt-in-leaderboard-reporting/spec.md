# opt-in-leaderboard-reporting Specification

## Purpose

Allow an operator to deliberately submit the current state of a
browser-imported managed character while preserving Gyrognome's default
credential-safe, transport-free operation.

## Requirements

### Requirement: Confirmed one-shot managed-character reporting

The system SHALL provide a foreground command that submits one profile-compatible
manual-brag (`t=b`) report for a selected managed character. It SHALL display
the safe identity, continuation profile, and realm and require explicit
interactive confirmation for each submission. Declining SHALL send no request
and leave state unchanged. Desktop confirmation SHALL NOT override eligibility.

#### Scenario: Confirming a report submission

- **WHEN** an operator confirms an eligible character's report
- **THEN** exactly one manual-brag request uses its persisted state and
  matching browser or desktop protocol

#### Scenario: Declining a report submission

- **WHEN** an operator declines confirmation
- **THEN** no request is sent and managed state is unchanged

### Requirement: Open the public leaderboard after manual brag

After a confirmed eligible manual brag completes its single report attempt,
the CLI and dashboard SHALL open the character's realm-specific leaderboard
using the fixed official HTTPS origin and safely encoded display name. The URL
SHALL exclude credentials, signed report data, and user-supplied endpoints.
Opening SHALL be detached and nonblocking; failure SHALL be reported
separately without changing the report outcome.

#### Scenario: Opening the leaderboard after a confirmed CLI brag

- **WHEN** an operator confirms an eligible character's manual brag from the CLI and its one report attempt completes
- **THEN** the system opens that character's realm-specific public leaderboard page with the display name safely encoded

#### Scenario: Opening the leaderboard after a dashboard brag

- **WHEN** an operator invokes an eligible character's Brag action in the dashboard and its one report attempt completes
- **THEN** the system opens that character's realm-specific public leaderboard page with the display name safely encoded

#### Scenario: Report delivery is rejected or fails

- **WHEN** the confirmed manual-brag report attempt completes with a rejected or failed delivery outcome
- **THEN** the public leaderboard page is still opened and the delivery outcome remains unchanged

#### Scenario: No confirmed manual brag occurs

- **WHEN** an operator declines the CLI confirmation, a manual action is blocked before submission, or a worker sends an automatic progress report
- **THEN** no public leaderboard page is opened

#### Scenario: The system browser cannot be opened

- **WHEN** the public leaderboard page cannot be opened through the system browser
- **THEN** the report outcome is preserved and a separate safe opening failure is reported

#### Scenario: Browser started fresh from the dashboard

- **WHEN** a dashboard brag opens the page while no browser is running and the newly started browser stays running and writes diagnostic output
- **THEN** the browser does not inherit Gyrognome's standard input, output, or error or share its terminal process group, none of its output appears in the terminal, the dashboard stays responsive, and a launcher still running after 3 seconds is treated as a successful open

#### Scenario: Launcher fails quickly in the dashboard

- **WHEN** the browser launcher exits unsuccessfully within 3 seconds of a dashboard brag
- **THEN** the dashboard shows a safe opening-failure message in its own interface and the report outcome is unchanged

### Requirement: Reporting eligibility and conformance gate

The system SHALL permit manual brag, motto, and guild actions only for eligible
imported online characters, active or inactive, based on their profile, import
provenance, credentials, and current evidence for the requested operation.
Local-only provenance SHALL prohibit online actions. Actions SHALL serialize
without interrupting runtime ownership. Ineligible requests SHALL be rejected
before transport with a safe reason.

#### Scenario: Reporting an offline-originated character

- **WHEN** an operator requests manual brag, motto, or guild action without a
  valid imported online credential
- **THEN** the character is reported ineligible and no request is sent

#### Scenario: Reporting an active managed character

- **WHEN** an otherwise eligible active character receives a manual-brag request
- **THEN** one report uses a coherent persisted character/profile snapshot
  through the serialized boundary without interrupting ownership

#### Scenario: Editing the profile of an active managed character

- **WHEN** an otherwise eligible active character receives a motto or guild action
- **THEN** the action uses the serialized boundary without interrupting ownership

#### Scenario: Conformance evidence is unavailable

- **WHEN** required evidence does not validate for the requested operation
- **THEN** the operation is refused without contacting the leaderboard

#### Scenario: Classic operation not covered by evidence

- **WHEN** desktop evidence covers another realm, credential mode, unapproved
  import path, or operation and no explicit guild readiness strategy applies
- **THEN** the requested action is ineligible even if browser evidence passes

#### Scenario: Reporting after the desktop spelling normalization

- **WHEN** a desktop import records only `load-spelling-patch`, its canonical
  and protocol equivalence is validated, and all other operation evidence
  matches
- **THEN** reporting eligibility is evaluated as the evidenced canonical path
  while the recorded adaptation remains visible

#### Scenario: Reporting after an unapproved adaptation

- **WHEN** a desktop import records a progression-affecting, unknown, or
  combined adaptation without separately matching evidence
- **THEN** the request is refused before transport even if one recorded
  adaptation would be eligible by itself

#### Scenario: Browser evidence does not authorize desktop reporting

- **WHEN** desktop reporting is requested with passing browser evidence but
  without matching desktop evidence
- **THEN** the request is refused before transport

### Requirement: Credential-safe report delivery

Requests SHALL use coherent persisted state and private authentication, and
only the profile's verified endpoint and contract. Redirects SHALL NOT leak
credentials or switch to unapproved or HTTP destinations. Logs and output
SHALL contain only safe identity, operation, eligibility reason, and outcome;
secrets, browser profiles, and raw or normalized responses SHALL be excluded.
HTTP success SHALL NOT prove leaderboard acceptance.

#### Scenario: Delivering an eligible report

- **WHEN** the approved endpoint returns successful HTTP status for manual brag
  or motto change
- **THEN** only a safe delivered outcome is reported, without a classification
  claim

#### Scenario: Categorizing a guild response

- **WHEN** a guild response matches a validated accepted or rejected
  fingerprint for the character's profile and realm
- **THEN** the matching safe category is exposed and no public fallback is
  required

#### Scenario: Reproducing a live-evidence fingerprint

- **WHEN** production classifies a response represented by a credential-free
  live-evidence vector
- **THEN** it uses the identical normalization version, ordered dynamic-value
  categories, replacement marker, and SHA-256 encoding

#### Scenario: Matching a guild response fingerprint

- **WHEN** a guild response is classified by fingerprint
- **THEN** it is bounded and matched only against sanitized fingerprints for
  that profile and realm using the same versioned normalization and dynamic
  value categories as live evidence

#### Scenario: Guild response fingerprint is unknown

- **WHEN** a bounded guild response matches no validated accepted or rejected
  fingerprint
- **THEN** the response outcome remains indeterminate unless the one permitted
  credential-free read of the verified realm page confirms the exact character
  and case-insensitive ASCII guild membership, and neither raw nor normalized
  response text is exposed or persisted

#### Scenario: Confirming Pemptus guild membership

- **WHEN** a supported Pemptus guild mutation has been attempted, including
  when delivery is uncertain
- **THEN** one bounded public membership observation determines confirmed
  desired state or safe uncertainty, without using response fingerprints or
  retrying the mutation

#### Scenario: Handling delivery failure

- **WHEN** delivery fails or returns unsuccessful HTTP status
- **THEN** a safe failure preserves state not explicitly locally persistent
  under the requested action and triggers no automatic retry

#### Scenario: Save contains an unapproved endpoint

- **WHEN** a desktop save supplies an arbitrary host, URL userinfo, or an
  endpoint with no verified HTTPS mapping
- **THEN** no credentials are transmitted and online eligibility explains the
  unsupported endpoint without echoing its raw value

### Requirement: No implicit or automated reporting

The system SHALL report only persisted level/act events from eligible active
online runtimes or explicit operator submissions for manual brag, motto, or
guild. Automatic delivery SHALL be one attempt with no queue or retry.
Registration, inspection, refresh, lifecycle, and administration SHALL NOT
submit reports. Ineligible events SHALL be discarded, never held for later
evidence.

#### Scenario: Advancing a managed online character

- **WHEN** an eligible active online character persists a level-up or act event
- **THEN** exactly one corresponding report is attempted without foreground
  confirmation

#### Scenario: Advancing a managed character

- **WHEN** a managed character advances through its worker or local lifecycle
- **THEN** only persisted level/act events from an eligible online worker can
  produce reports

#### Scenario: Advancing a managed offline character

- **WHEN** an active character without an online credential advances
- **THEN** no leaderboard request is submitted

#### Scenario: Refreshing a dashboard

- **WHEN** the dashboard refreshes without an explicit online action
- **THEN** it sends no leaderboard request

#### Scenario: Gated desktop advancement

- **WHEN** an ineligible desktop character crosses a reporting boundary
- **THEN** it advances locally, records required local-only provenance, and
  neither sends nor queues the report

#### Scenario: Leaving a guild unchanged

- **WHEN** an operator submits an empty guild value
- **THEN** no guild mutation request is sent

### Requirement: Pemptus desktop request contract

Pemptus requests SHALL require a valid passkey, empty account/password fields,
realm `Pemptus`, and the exact saved endpoint
`http://progressquest.com/pemptus.php?`, delivered to
`https://progressquest.com/pemptus.php` without an Authorization header.
Unsupported destinations, credentials, or encoding SHALL fail before transport.

#### Scenario: Sending an eligible Pemptus manual brag

- **WHEN** an operator requests manual brag for a fresh supported Pemptus
  managed import with matching manual-operation evidence
- **THEN** the system attempts one desktop-compatible report to the fixed
  Pemptus HTTPS endpoint without an Authorization header

#### Scenario: Missing unrelated operation evidence

- **WHEN** a supported Pemptus import has passing manual-operation evidence
  but lacks guild or automatic-operation evidence and has not advanced
- **THEN** manual brag remains eligible and the missing operations remain
  individually gated

#### Scenario: Rejecting a mismatched Pemptus contract

- **WHEN** a Pemptus request uses a different saved endpoint, non-empty account
  or password, unsupported encoding, or mismatched evidence
- **THEN** the requested action is blocked with a credential-safe reason
  before an HTTP request is attempted

### Requirement: Pemptus operation-specific evidence

Current passing evidence SHALL cover the requested operation, import path,
encoding, and authentication contract. Manual brag, automatic level, and
automatic act reporting SHALL be gated independently. Inspection, runtime, and
callers SHALL agree; evidence for another realm, credential mode, or operation
SHALL NOT confer eligibility.

#### Scenario: Reporting persisted Pemptus progress

- **WHEN** an eligible Pemptus runtime persists a level or act transition
- **THEN** it attempts the corresponding report after persistence using the
  current serialized profile, without a retry or deferred report queue

### Requirement: Pemptus import-family evidence

Unadapted, load-spelling-only, and exact indexed quest-placeholder-only Pemptus
imports MAY share explicitly approved readiness evidence. Exact evidence SHALL
take precedence; stale, invalid, inconclusive, or duplicate exact evidence
SHALL NOT be bypassed. Equivalent evidence SHALL retain its observed adaptation
path. Unsupported or combined adaptations SHALL remain ineligible, and original
import provenance SHALL remain authoritative after runtime marker resolution.

#### Scenario: Enabling automatic reporting across the supported family

- **WHEN** current valid placeholder level/act records cover a supported
  unadapted, spelling-only, or placeholder-only Pemptus import
- **THEN** inspection and runtime agree that automatic reporting is eligible
  without changing the records' declared observation paths

#### Scenario: Reusing manual and motto coverage for supported placeholder imports

- **WHEN** current valid unadapted Pemptus manual/motto records cover an otherwise
  eligible placeholder-only import through the approved policy
- **THEN** manual and motto actions are available without relabeling those records

#### Scenario: Supporting an existing unadvanced Izot-equivalent import

- **WHEN** an existing unadapted or spelling-only Pemptus managed import with
  valid credentials and no local-only history receives the revised readiness policy
- **THEN** it becomes eligible without reimport, and normal advancement does not
  fork local-only solely because its original path differs from automatic evidence

#### Scenario: Retaining adapted-import provenance after marker resolution

- **WHEN** an eligible placeholder-only runtime resolves its indexed marker
  during ordinary task completion
- **THEN** later eligibility still uses its recorded placeholder import path,
  not an invented unadapted history

#### Scenario: Keeping combined adaptations outside placeholder coverage

- **WHEN** an import combines quest-placeholder with spelling correction or
  legacy prologue despite a passing placeholder automatic record
- **THEN** the combined path remains ineligible before transport

### Requirement: Pemptus persistence and compatibility

Pemptus reporting SHALL preserve persistence, serialization, single-attempt,
callback timing, and durable local-only provenance. Advancement while automatic
level or act reporting is gated SHALL permanently mark the timeline local-only;
missing only manual or profile-action evidence SHALL NOT. Browser reporting and
valid Spoltog reporting SHALL retain their existing behavior.

#### Scenario: Advancing before progress evidence is complete

- **WHEN** a Pemptus managed import first advances while level or act reporting
  is gated
- **THEN** advancement and local-only provenance are persisted atomically and
  later passing evidence cannot reconnect that timeline

#### Scenario: Advancing independently of guild verification

- **WHEN** all other operations, including level and act reporting, are eligible
  but a guild action's public confirmation fails
- **THEN** the runtime retains online provenance and automatic reporting remains
  available without treating the guild result as an automatic-evidence failure

#### Scenario: Preserving existing reporting paths

- **WHEN** browser imports or Spoltog account/password imports use their
  currently valid supported contracts
- **THEN** request construction, authentication, explicit-action behavior, and
  automatic-report ordering remain unchanged

#### Scenario: Preserving local-only permanence when placeholder coverage changes

- **WHEN** a placeholder-only managed import advances without either automatic
  record and matching records are installed afterward
- **THEN** its local-only provenance remains permanent and all online actions
  stay blocked
