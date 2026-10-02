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

After an eligible manual brag has been explicitly confirmed and its single
report attempt has completed, the system SHALL open the selected character's
public Progress Quest leaderboard page through the system browser. The page
SHALL correspond to the character's supported realm and identify the character
by its display name, safely encoded as a query value. The page URL SHALL use a
fixed official HTTPS origin and SHALL NOT contain credentials, signed report
data, or user-supplied endpoint data. This behavior SHALL apply to both the
confirmed CLI report command and the dashboard Brag action, regardless of the
categorized report-delivery outcome. Failure to open the page SHALL be reported
separately and SHALL NOT change the report outcome.

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

### Requirement: Reporting eligibility and conformance gate

The system SHALL allow manual brag, motto, and guild actions for valid imported
online browser characters with retained passkeys while active or inactive when
applicable evidence passes. Desktop characters SHALL additionally require
matching profile, import-path, realm, endpoint, credential-mode, encoding, and
operation evidence or the explicit Pemptus public-confirmation guild strategy,
and SHALL NOT have local-only advancement provenance.
For import-path matching, an explicitly proven deterministic load
normalization MAY match its canonical path only under the equivalence rule
defined by desktop leaderboard conformance. The separately approved Pemptus
family also permits exact indexed quest-placeholder-only imports. Unknown,
unapproved progression-affecting, or combined adaptations do not inherit eligibility.
Each explicit action SHALL serialize with other online actions without
stopping, restarting, or interrupting runtime ownership.

Offline origins, invalid/missing credentials, unsupported authentication or
endpoints, local-only provenance, and incomplete evidence SHALL be rejected
before transport, with a safe reason. Valid browser evidence SHALL NOT confer
desktop eligibility.

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

### Requirement: Credential-safe report delivery

The reporting surface SHALL construct profile-compatible manual-brag,
motto-change, and guild requests from coherent persisted state, online profile,
and retained private authentication. Browser delivery SHALL retain its existing
official endpoint behavior. Desktop delivery SHALL use only an explicitly
verified allowlisted official HTTPS endpoint and authentication contract; a
saved host or URL userinfo SHALL NOT authorize a destination. Redirects SHALL
NOT leak authentication or switch to an unapproved endpoint or HTTP.

Output SHALL contain only safe identity, operation, eligibility reason, and
categorized outcome. Logs/errors SHALL NOT contain passkeys, account
logins/passwords, signed or authenticated URLs, raw saves/responses, browser
profiles, or normalized response text. Successful HTTP status SHALL mean
delivered, not proof of normal leaderboard classification.

Fingerprint-based guild responses SHALL be bounded and matched only to sanitized
profile-and-realm-specific fingerprints produced by the same versioned
normalization contract as the live evidence. The production classifier SHALL
supply the same ordered categories of dynamic values and use the same
replacement marker as the evidence generator. After an indeterminate desktop
guild response, the same explicit action MAY perform one bounded,
credential-free read of the verified realm page and treat an exact character
and case-insensitive ASCII guild match as acceptance without retrying the guild
mutation.

Supported Pemptus guild actions SHALL instead require one bounded public
membership observation after the sole mutation attempt, including uncertain
delivery. They SHALL NOT use response fingerprints as acceptance or rejection
proof; confirmed desired state, safe uncertainty, and persistence SHALL follow
the Pemptus profile-action requirement.

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

#### Scenario: Guild response fingerprint is unknown

- **WHEN** a bounded guild response matches no validated accepted or rejected
  fingerprint
- **THEN** the response outcome remains indeterminate unless the one permitted
  public-profile observation verifies the requested membership, and neither
  raw nor normalized response text is exposed or persisted

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

The system SHALL submit profile-compatible requests only for persisted
level-up/act-completion events from an eligible active online runtime or an
operator's explicit manual-brag, motto-change, or guild-designation submission.
Empty guild means leave. Automatic delivery SHALL be one attempt, never queued
or retried. Registration, inspection, dashboard refresh, lifecycle operations,
and administration SHALL NOT independently submit requests. Ineligible desktop
events SHALL be discarded for delivery, not held for a later evidence update.

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

### Requirement: Evidence-scoped Pemptus desktop reporting

The system SHALL support reporting from managed Pemptus desktop-6.4.4 imports
when current passing evidence covers the requested operation through exact or
explicitly approved equivalent import coverage, encoding, and authentication
contract. The Pemptus
contract SHALL require a valid passkey, empty account/password fields, the exact
realm `Pemptus`, and the exact saved endpoint
`http://progressquest.com/pemptus.php?`, delivered to
`https://progressquest.com/pemptus.php` without an Authorization header.

Manual brag, automatic level reporting, and automatic act reporting SHALL be
gated independently. Inspection and runtime SHALL agree on those gates for
equivalent inputs. Neither an endpoint mapping nor evidence for another realm,
credential mode, or operation SHALL confer eligibility. Unsupported
destinations, credentials, and encoding SHALL fail before transport.

Unadapted, load-spelling-only, and exact indexed quest-placeholder-only Pemptus
imports SHALL share the explicitly approved readiness family for automatic and
manual reporting. Inspection, runtime, and requested-operation callers SHALL
agree, including previously imported unadvanced records. Exact matching records
SHALL take precedence; stale, invalid, inconclusive, or duplicate matching
records SHALL NOT be bypassed with equivalent coverage. Selected evidence SHALL
retain its true observed adaptation path. Original import provenance SHALL
remain authoritative after existing runtime marker resolution. Unsupported
or combined adaptations SHALL remain excluded.

Pemptus reporting SHALL retain the existing persistence, serialization,
no-retry, callback timing, and durable local-only provenance contracts.
Advancement with either automatic progress operation gated SHALL permanently
mark the managed timeline local-only; missing only manual or profile-action
evidence SHALL NOT do so. Browser reporting and valid Spoltog
account/password reporting SHALL retain their existing behavior.

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

#### Scenario: Reporting persisted Pemptus progress

- **WHEN** an eligible Pemptus runtime persists a level or act transition
- **THEN** it attempts the corresponding report after persistence using the
  current serialized profile, without a retry or deferred report queue

#### Scenario: Rejecting a mismatched Pemptus contract

- **WHEN** a Pemptus request uses a different saved endpoint, non-empty account
  or password, unsupported encoding, or mismatched evidence
- **THEN** the requested action is blocked with a credential-safe reason
  before an HTTP request is attempted

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

#### Scenario: Preserving local-only permanence when placeholder coverage changes

- **WHEN** a placeholder-only managed import advances without either automatic
  record and matching records are installed afterward
- **THEN** its local-only provenance remains permanent and all online actions
  stay blocked
