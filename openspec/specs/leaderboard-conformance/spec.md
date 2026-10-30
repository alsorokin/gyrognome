# leaderboard-conformance Specification

## Purpose

Establish evidence that Gyrognome's locally advanced characters produce
browser-equivalent leaderboard report histories before general reporting exists.

## Requirements

### Requirement: Browser-equivalent report traces

The system SHALL derive an ordered, credential-free leaderboard report trace
from deterministic character advancement and explicitly requested browser
actions. Each trace entry SHALL identify the browser trigger and the complete
canonical snapshot at the point that the browser emits that report. The trace
SHALL preserve the browser's event order when a single advancement crosses
multiple report-producing transitions.

#### Scenario: Capturing a level-up report snapshot

- **WHEN** an advancement completes a task that reaches a level boundary
- **THEN** the derived trace contains an `l` entry with the canonical snapshot
  after the browser's level-up effects and before later completion effects

#### Scenario: Capturing an act-completion report snapshot

- **WHEN** an advancement completes an act
- **THEN** the derived trace contains an `a` entry at the browser-equivalent
  act-completion point

#### Scenario: Capturing explicit browser actions

- **WHEN** a caller requests a manual brag or changes a motto in the conformance
  input
- **THEN** the derived trace contains respectively a `b` or `m` entry with the
  corresponding browser-equivalent canonical snapshot

### Requirement: Credential-free trace conformance

The system SHALL verify browser-derived report traces using disposable,
synthetic character observations. A trace fixture SHALL record event order,
trigger, canonical snapshot, expected unsigned request fields, normalized
request representation, and validator result using a synthetic passkey.
Committed fixtures and diagnostics SHALL NOT contain player saves, live
passkeys, browser profiles, or complete signed leaderboard URLs.

#### Scenario: Replaying a browser-derived trace

- **WHEN** the system replays a committed report-trace fixture
- **THEN** every derived event's trigger, snapshot, unsigned fields,
  normalized representation, and synthetic validator match the fixture exactly

#### Scenario: Rejecting sensitive trace data

- **WHEN** a report-trace fixture or diagnostic artifact contains a player save,
  live passkey, browser profile, or complete signed request URL
- **THEN** the system rejects it before it can be committed or displayed

### Requirement: Controlled anti-cheat conformance evidence

The project SHALL define an explicitly confirmed Playwright procedure using a
new disposable official-browser character. It SHALL compare browser and
Gyrognome report histories, exercise accepted non-empty, accepted empty, and
invalid guild submissions, use a shared versioned credential-safe fingerprint
contract, and require normal-population evidence before enabling production
actions.

#### Scenario: Running an external conformance experiment

- **WHEN** an operator explicitly confirms a disposable-character experiment
- **THEN** the harness creates the character through the official browser,
  sends no reports or guild requests for any existing managed character, and
  records credential-free comparison, guild-outcome, and
  leaderboard-classification evidence for the disposable character, restoring
  it to no guild before completion

#### Scenario: Observing accepted and rejected guild submissions

- **WHEN** the disposable browser character submits a testable existing guild
  designation, submits a deliberately invalid designation, and then submits an
  empty designation
- **THEN** the evidence records sanitized request field names, operation order,
  safe accepted or rejected categories, and normalized response fingerprints
  produced by the shared versioned contract, without recording a private guild
  designation, passkey, credential, signed URL, or raw response body

#### Scenario: Replaying an evidence fingerprint in production

- **WHEN** a credential-free response vector is fingerprinted by the live
  evidence path and classified by the production guild path with corresponding
  dynamic values
- **THEN** both paths produce the same fingerprint and classification

#### Scenario: Guild response fingerprints are ambiguous

- **WHEN** normalized accepted and rejected guild responses do not produce
  distinct credential-safe fingerprints
- **THEN** guild conformance fails and production guild actions remain disabled

#### Scenario: Guild cleanup cannot be confirmed

- **WHEN** the harness cannot confirm that the disposable character has no
  guild after the empty designation is submitted
- **THEN** guild conformance fails and production guild actions remain disabled

#### Scenario: Evidence is incomplete or classified as cheating

- **WHEN** a required report or guild scenario lacks passing evidence or the
  disposable character is classified in the cheater population
- **THEN** the conformance gate fails and the corresponding general online
  operations remain unsupported

#### Scenario: Comparing the complete browser report history

- **WHEN** the approved browser experiment runs
- **THEN** it compares initial load, pause, restart, delayed callbacks, task
  completion, level-up, act completion, manual brag, and motto change; the
  passkey remains in the ephemeral experiment and native `newguy` support is
  not required

#### Scenario: Normalizing guild response fingerprints

- **WHEN** evidence and production classify guild responses
- **THEN** both use the same versioned normalization, replacing the same
  ordered identity, authentication, passkey, and prior/submitted guild values
  before hashing, with credential-free vectors proving cross-path agreement

### Requirement: Source-derived desktop local conformance

Local conformance SHALL compare production against a separately authored
reference derived from pinned desktop 6.4.4 source. Credential-free evidence
SHALL identify source/build and implementation identities, synthetic starting
and random state, inputs, transitions, resulting state, random continuation,
request representation, and unverified Delphi edges. Coverage SHALL include
supported loading, progression, callbacks, and requests while preserving
browser conformance.

#### Scenario: Replaying a desktop checkpoint

- **WHEN** a synthetic desktop checkpoint is replayed
- **THEN** state, ordered transitions, random continuation, and unsigned report
  representation match the pinned source-derived reference observation exactly

#### Scenario: Missing official-runtime corroboration

- **WHEN** the production implementation and source-derived reference agree but
  exact Delphi random or rounding behavior remains uncorroborated
- **THEN** local continuation may use the documented source-derived contract,
  but the limitation remains visible, no exact-runtime claim is made, and the
  evidence cannot enable a classic online operation

#### Scenario: Verifying legacy adaptation

- **WHEN** a 6.2-shaped synthetic save is loaded by the source-derived reference
  harness and the importer
- **THEN** load-time state and subsequent callback transitions match, including
  legacy queues, quest placeholders, and spell collection-order effects, while
  any missing official-runtime corroboration remains disclosed

#### Scenario: Covering desktop source-derived edge cases

- **WHEN** local conformance fixtures are generated and replayed
- **THEN** coverage includes supported 6.2 adaptations, fresh prologues,
  legacy markers, spelling patches, weighted randomness, numeric assumptions,
  rounding,
  fractional-second maxima, full-bar callbacks, pause/restart/delay,
  market/noncombat, level-up, later acts, and manual/motto/guild requests;
  byte encoding, omitted fields, and synthetic validator results match, and
  fixtures contain no real saves, credentials, authenticated URLs, or raw
  responses

### Requirement: Scoped classic-realm live evidence

Except for the explicit Pemptus readiness contract, classic operations SHALL
require matching passing evidence for desktop profile, import path,
realm/endpoint, credential mode, encoding, operation, and implementation
identity. Live tests require explicit bounded disposable scope, exclusive
client handoff, real-time progression, sanitized evidence, and normal
classification; relevant behavior changes invalidate affected evidence.

#### Scenario: No approval for live experimentation

- **WHEN** planning or implementation is authorized but a scoped live
  experiment is not
- **THEN** no online character is created or reported by the experiment and
  the experiment confers no additional production readiness

#### Scenario: Approving a classic live experiment

- **WHEN** a classic live experiment is approved
- **THEN** approval names the realm, disposable account/character, operations,
  and bounds; a new official-desktop character is used unless the explicit
  Pemptus immediate-stage exception applies, credentials stay ephemeral, and
  client handoff prevents simultaneous reporters

#### Scenario: Browser evidence offered for a desktop character

- **WHEN** an operation has passing browser evidence but no matching desktop
  realm/profile evidence
- **THEN** the desktop gate refuses it without contacting the server

#### Scenario: Distinct diagnostic motto after an inconclusive observation

- **WHEN** an approved live experiment has an inconclusive motto observation
  and the operator separately approved another distinct motto within the bound
- **THEN** the runner may submit the distinct intent once, does not replay the
  earlier exact intent, and retains only sanitized intent and response evidence

#### Scenario: An incomplete classic result

- **WHEN** authentication, a required scenario, or normal classification cannot
  be established for the approved disposable character
- **THEN** the result is inconclusive or failed, never passing, and cannot
  enable the affected production operation

#### Scenario: Enabling only evidenced operations

- **WHEN** manual reporting passes for one realm/credential mode but guild
  behavior or a legacy adaptation path remains unverified
- **THEN** only the covered operation and import path become eligible unless
  the separate explicit Pemptus equivalent-coverage or public-confirmation
  policy applies; unapproved operations and paths remain gated

#### Scenario: Applying a proven spelling normalization

- **WHEN** a supported desktop import records only `load-spelling-patch` and
  differential conformance proves its post-load canonical state and request
  construction match the evidenced canonical path
- **THEN** the import may use the matching realm, credential-mode, and operation
  evidence while retaining the adaptation in its visible provenance

#### Scenario: Rejecting a substantive or combined adaptation

- **WHEN** an import records a legacy prologue, legacy quest placeholder,
  unknown adaptation, or `load-spelling-patch` combined with another adaptation
  without separately matching live evidence or the explicit Pemptus family policy
- **THEN** classic online operations remain ineligible without contacting the
  endpoint

#### Scenario: Bounding classic progression requests

- **WHEN** an approved classic progression stage runs
- **THEN** it uses real active time without accelerated state or report replay,
  records each distinct level transition before the first act as its own
  intent, attempts the first act at most once, and never replays an
  inconclusive exact request

### Requirement: Bounded Pemptus manual-report diagnostic

The project SHALL provide a development-only, explicitly approved one-shot
native manual-report diagnostic for a disposable Pemptus import. It SHALL
enforce the exact import, realm, endpoint, credential, and ASCII contract,
send only to the HTTPS endpoint without authentication, redirects, downgrade,
or retries, and preserve source state without registering or modifying the
character.

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

#### Scenario: Running validation without network access

- **WHEN** the diagnostic runs in validation-only mode
- **THEN** it contacts no server; live mode uses only the exact saved endpoint,
  positive passkey, empty account/password, supported ASCII request, no
  Authorization header, and no redirect or HTTP downgrade

#### Scenario: Restricting diagnostic scope

- **WHEN** an approved diagnostic executes against an imported character
- **THEN** the source is a supported desktop-6.4.4 canonical import established
  through the official desktop client, the saved endpoint is exactly
  `http://progressquest.com/pemptus.php?`, delivery targets only
  `https://progressquest.com/pemptus.php`, and no progression, managed
  registration, motto/guild change, source mutation, or credential/request
  persistence occurs

### Requirement: Honest and credential-safe diagnostic observations

The diagnostic SHALL make bounded, credential-free public observations of the
exact character before and for at most 60 seconds after submission, with
bounded requests and responses. Output SHALL distinguish delivery, rejection,
classification, and state agreement; ambiguous or unchanged public state is
inconclusive. Results SHALL exclude sensitive data and SHALL NOT enable
production operations.

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

#### Scenario: Bounding and sanitizing observations

- **WHEN** the diagnostic observes public state
- **THEN** each request and wait fits the remaining 60-second deadline, response
  sizes are limited, and output excludes passkeys, credentials, signed URLs,
  save contents, and raw or normalized response bodies; malformed data,
  ambiguous pages, size-limit failures, and timeouts remain inconclusive

### Requirement: Independent classic operation evidence

Classic eligibility SHALL validate current, integrity-valid evidence for the
requested realm, endpoint, credential mode, import path, encoding, and
operation independently. Guild fingerprints are guild-only evidence; invalid
or missing guild evidence SHALL NOT invalidate non-guild evidence, and
evidence SHALL NOT be copied across contracts.

#### Scenario: Validating manual evidence without guild fingerprints

- **WHEN** a current integrity-valid manual-operation record satisfies every
  matching scope, acceptance, classification, and cleanup requirement but
  contains no guild-response fingerprints
- **THEN** manual evidence validation succeeds without enabling guild actions

#### Scenario: Isolating invalid guild evidence

- **WHEN** independently valid level, act, manual, or motto evidence exists
  alongside invalid guild-only evidence
- **THEN** the non-guild evidence remains valid and a fingerprint-based guild
  strategy cannot use that invalid record

#### Scenario: Rejecting evidence outside its contract

- **WHEN** an evidence record differs from the requested realm, endpoint,
  credential mode, operation, approved import coverage, implementation/source identity,
  integrity, or validity period
- **THEN** it cannot enable the requested operation

#### Scenario: Retaining unchanged Spoltog coverage

- **WHEN** existing Spoltog evidence passes validation and its original
  evidence-relevant behavior is unchanged
- **THEN** its covered account/password operations remain eligible without
  granting Pemptus coverage

#### Scenario: Requiring complete guild-only fingerprint coverage

- **WHEN** fingerprint-based guild evidence is used
- **THEN** matching join, change, leave, and rejection coverage uses the shared
  normalization contract; Pemptus public-confirmation readiness does not
  require those historical fingerprints

### Requirement: Usable Pemptus delivery under explicit equivalence

Otherwise eligible Pemptus imports on unadapted, spelling-only, or exact
supported placeholder-only paths SHALL support five operations through narrow
evidence reuse and bounded public guild confirmation. The policy SHALL preserve
source provenance and local-only history; completion requires cross-path
behavior and regression checks, not another live campaign or exhaustive
certification.

#### Scenario: Delivering all five operations on each supported path

- **WHEN** current valid bundled records and the public-confirmation strategy
  apply to an otherwise eligible Pemptus import on any supported path
- **THEN** all five gates are eligible, with original adaptation provenance intact

#### Scenario: Completing usable support without another live campaign

- **WHEN** supported-path behavior and focused preservation regressions pass
- **THEN** this change can finish without repeating completed cases or claiming
  that every path/subcase received independent live certification

#### Scenario: Preserving the historical guild audit gap

- **WHEN** a supported guild action uses public confirmation but historical
  response fingerprints remain absent
- **THEN** the action is available without inventing a guild evidence record
  or retrospectively marking the fingerprint audit conclusive

#### Scenario: Refusing arbitrary evidence inheritance

- **WHEN** an import uses another realm, unknown adaptation, or combined path
- **THEN** the Pemptus family policy does not confer eligibility

#### Scenario: Limiting equivalent Pemptus coverage

- **WHEN** a Pemptus import uses an unadapted, spelling-only, or exact
  placeholder-only path
- **THEN** the five operations are supported only with current valid evidence
  reused without relabeling or cross-realm inheritance; the placeholder path
  also requires a valid saved pinned-table index

### Requirement: Non-enabling offline adaptation studies

Offline placeholder-equivalence study SHALL cover only the exact indexed
`fQuest` placeholder in an otherwise supported state, using verified pinned
source semantics and credential-free synthetic pairs for state, RNG,
callback/report order, and all five unsigned requests. Findings SHALL NOT
change eligibility, relabel evidence, or authorize live requests; broader,
failed, or inconclusive cases remain gated.

#### Scenario: Establishing offline equivalence without live authorization

- **WHEN** synthetic comparisons establish the narrowly defined placeholder
  equivalence under verified source semantics
- **THEN** the result is recorded for review without enabling an operation,
  inheriting bundled evidence, authorizing live execution, or satisfying real
  active-time requirements

#### Scenario: Rejecting a broader normalization claim

- **WHEN** a candidate has a mismatched index or caption, unsupported task form,
  legacy prologue, or another substantive/combined adaptation
- **THEN** the placeholder study cannot confer an adaptation exemption or
  production eligibility on that candidate

#### Scenario: Recording failed or inconclusive equivalence

- **WHEN** canonical state, RNG use, callback/report order, or unsigned request
  shapes differ, or source semantics do not conclusively establish equivalence
- **THEN** the result remains failed or inconclusive and progression stays
  blocked without a validation bypass or invented evidence

#### Scenario: Keeping offline proof artifacts credential-free

- **WHEN** paired states or observations are retained for the offline study
- **THEN** they contain synthetic data only, preserve adaptation provenance, and
  do not contain a real save, credential, or signed leaderboard request

### Requirement: Explicit placeholder-only progression conformance

The development runner MAY admit only a freshly approved Pemptus progression
source with the exact indexed placeholder as its sole adaptation and the
`--allow-quest-placeholder` flag. It SHALL bind original source/state/path and
flag to checkpoint scope and preserve provenance. Exact-path records take
precedence; only their absence permits approved equivalent fallback, while
invalid or duplicate matches fail closed. This policy SHALL NOT authorize live
execution.

#### Scenario: Validating an explicitly scoped placeholder-only source

- **WHEN** a freshly approved Pemptus progression source has the exact indexed
  placeholder as its sole adaptation and the explicit flag is present
- **THEN** offline validation can accept that path while retaining original
  provenance, without sending requests or emitting enabling evidence

#### Scenario: Refusing implicit or broader placeholder admission

- **WHEN** the flag is absent, the stage/realm is wrong, the marker/index is
  invalid, authorization is missing, the caption/task form is unsupported, or
  the source combines placeholder with spelling/prologue adaptation
- **THEN** the runner rejects placeholder admission before network activity

#### Scenario: Refusing a changed placeholder checkpoint scope

- **WHEN** an input source, saved index, adaptation path, or admission flag
  differs from the bound checkpoint scope
- **THEN** continuation fails without replaying a request or rewriting provenance

#### Scenario: Emitting actual adapted automatic evidence

- **WHEN** approved placeholder-only automatic cases pass live acceptance,
  normal classification, and final cleanup
- **THEN** their new records declare `[legacy-quest-placeholder]`, not an empty
  adaptation array inferred from the later canonical callback state; only
  Pemptus automatic level/act records with the required identity, integrity,
  freshness, acceptance, and cleanup can validate

#### Scenario: Selecting independently pinned records for distinct paths

- **WHEN** valid unadapted and placeholder records exist for the same automatic
  operation
- **THEN** each import prefers its matching path,
  and duplicate records for one matching scope fail closed

#### Scenario: Refusing equivalent fallback after matching-record expiry

- **WHEN** the requested placeholder record exists but is invalid, stale, or
  inconclusive while another path's operation record passes
- **THEN** that other record cannot enable the placeholder operation

#### Scenario: Reusing approved equivalent coverage when an exact record is absent

- **WHEN** a supported Pemptus import lacks an exact-path operation record but
  has one unambiguous current valid record on an approved equivalent path
- **THEN** that operation is eligible without altering the record's observed
  path or the import's original provenance

#### Scenario: Rejecting placeholder records outside the automatic scope

- **WHEN** a candidate placeholder record claims manual brag, motto, guild,
  another realm, or a combined adaptation path
- **THEN** validation rejects it without weakening independently valid records

#### Scenario: Preserving task-resolution and handoff constraints

- **WHEN** the development runner admits a placeholder-only progression source
- **THEN** it requests both automatic level and act operations, validates the
  exact original marker and saved index before task resolution, retains normal
  realm/authentication handoff checks, and changes neither the real save nor
  importer/task normalization

### Requirement: Separately authorized Pemptus live conformance

The development procedure SHALL support the native Pemptus passkey-only path.
Live stages SHALL require explicit bounded approval, disposable-character
scope, exclusive client handoff, attributable observations, normal
classification, and cleanup.



#### Scenario: Running local validation without live approval

- **WHEN** Pemptus conformance tooling is exercised without the required live
  confirmations
- **THEN** no leaderboard mutation occurs and no production-passing evidence
  is emitted

#### Scenario: Proving manual-report consumption

- **WHEN** an approved Pemptus manual report produces a uniquely attributable
  observable effect under the validated acceptance procedure
- **THEN** the procedure can record manual-operation acceptance while checking
  normal classification, without treating that result as motto or guild proof

#### Scenario: Requesting an existing-character exception without preparation approval

- **WHEN** an operator requests a preliminary guild leave without an explicitly
  approved existing-disposable immediate scope and sufficient attempt bound
- **THEN** validation fails before any mutation or enabling output

#### Scenario: Rejecting a stale copied baseline

- **WHEN** an approved older disposable save differs from the normal public
  report fields or current guild
- **THEN** the stage stops before its preparatory leave or requested mutations

#### Scenario: Using an approved empty control baseline

- **WHEN** the saved and public control motto are both empty and the immediate
  exception approves that baseline with distinct non-empty native markers
- **THEN** manual consumption still requires its attributable marker effect
  and does not grant motto coverage from baseline agreement

#### Scenario: Preparing an already-guilded disposable character

- **WHEN** a matching existing-guild baseline is approved for one preparatory
  leave and seven requested immediate mutations
- **THEN** all eight attempts share the approved cap, preparation is recorded
  before delivery, and join/change/rejection/final-leave coverage remains required

#### Scenario: Interrupting the preparatory leave

- **WHEN** the preparatory leave has no conclusive recorded observation
- **THEN** continuation cannot replay it and no operation record is enabling

#### Scenario: Preserving the exploratory originals

- **WHEN** an approved existing-character immediate stage completes
- **THEN** final no-guild state and private-copy cleanup are confirmed without
  deleting or modifying the original source save or backup

#### Scenario: Authorizing an existing-character exception

- **WHEN** an older disposable character is approved for the immediate stage
- **THEN** it passes fresh level-1 handoff checks, matches the normal public and
  saved report baseline and current guild, and uses an exclusive stopped-client
  handoff; the exception authorizes no official-client mutation

#### Scenario: Recognizing a declared but empty trailing guild column

- **WHEN** a Pemptus table explicitly declares the trailing Guild column and
  its unique exact-name row contains every preceding cell but omits only that
  empty trailing cell
- **THEN** realm-aware observation can confirm no guild without interpreting
  an absent header, malformed alignment, or unrelated row as empty membership

#### Scenario: Reconciling preparation without replay

- **WHEN** a freshly approved 60-second read-only window verifies an unchanged
  source/scope, the sole recorded preparatory intent, normal report fields, and
  no-guild state
- **THEN** preparation can be recorded as reconciled without resending it,
  fabricating its missing response fingerprint, or granting operation coverage

#### Scenario: Continuing the remaining cases after reconciliation

- **WHEN** approved preparation reconciliation succeeds
- **THEN** only the seven still-unattempted primary cases may run under the
  original eight-attempt total cap with fresh bounded observations and final cleanup

#### Scenario: Rejecting unsupported reconciliation

- **WHEN** recovery targets an unknown primary operation, a different source
  or scope, callback progress, multiple attempts, abnormal classification, or
  an unconfirmed or expired public observation
- **THEN** no mutation is replayed, continuation stays blocked, and no
  production-enabling evidence is emitted

#### Scenario: Observing an unchanged row after HTTP success

- **WHEN** a manual request returns HTTP 200 with an empty body and the public
  row is unchanged from the baseline
- **THEN** delivery and state agreement are recorded separately from acceptance
  and no passing manual evidence is emitted from those observations alone

#### Scenario: Continuing an operator-accepted case-only guild join

- **WHEN** the explicitly approved five-attempt checkpoint matches the original
  scope and join intent, the operator confirms no competing request, and a fresh
  bounded read-only check verifies normal report fields and linked `BEERguild`
- **THEN** separate operator acceptance can skip that already attempted join
  without replay and permit only guild change, rejection, and final leave under
  the original eight-attempt cap

#### Scenario: Preserving absent join evidence during continuation

- **WHEN** operator-accepted join continuation completes remaining cases and
  confirmed cleanup but the original join response fingerprint is absent
- **THEN** the runner explicitly excludes guild production evidence while
  independently complete manual and motto records can pass validation

#### Scenario: Refusing a broader operator-acceptance recovery

- **WHEN** continuation encounters a changed scope/source, wrong intent,
  different uncertain primary case, callback progress, unconfirmed membership,
  abnormal classification, or an expired fresh observation
- **THEN** it stops without replay, further mutation, or invented evidence

#### Scenario: Completing immediate coverage without progression coverage

- **WHEN** approved immediate operations pass and their cleanup is confirmed
  but automatic level/act coverage has not passed
- **THEN** only the proven immediate operations can receive enabling evidence
  and absent valid automatic coverage leaves usable Pemptus support unfinished

#### Scenario: Bounding a progression experiment

- **WHEN** an approved native progression stage reaches its active-time bound
  without observing all requested transitions or their acceptance
- **THEN** unproven operations remain inconclusive, no progression is
  accelerated, and the operator receives a credential-safe bounded outcome

#### Scenario: Resuming after an uncertain mutation

- **WHEN** a checkpoint records a mutation attempt without a conclusive
  corresponding observation
- **THEN** the stage cannot replay that intent to obtain passing evidence

#### Scenario: Failing cleanup or classification

- **WHEN** private-artifact or guild cleanup cannot be confirmed, or the
  character is classified outside the normal population
- **THEN** the affected stage does not produce production-enabling evidence

#### Scenario: Attempting to promote the archived probe

- **WHEN** the archived Pemptus diagnostic result is presented as operation
  evidence
- **THEN** it fails the production-enabling contract rather than being upgraded
  merely because HTTP delivery succeeded

#### Scenario: Preserving Pemptus table and guild identity semantics

- **WHEN** public Pemptus guild state is parsed or reconciled
- **THEN** only an explicitly declared empty trailing Guild cell in a uniquely
  aligned exact-name row can mean no guild; malformed or ambiguous rows remain
  unknown, production and development readers agree, Spoltog semantics stay
  unchanged, guild names compare ASCII-case-insensitively while retaining
  observed spelling, and character identity remains exact

#### Scenario: Recording and bounding every live mutation

- **WHEN** an approved Pemptus operation is attempted
- **THEN** the attempt is recorded before mutation, independently attributable
  acceptance and normal classification are required, HTTP success alone is
  inconclusive, and only credential-free records for the proven operation and
  path with confirmed cleanup can enable production

### Requirement: Rested runtime pacing readiness exception

Managed production runtimes SHALL apply rested progression to otherwise
eligible online characters without requiring new leaderboard evidence for
timing alone, provided non-pacing simulation, callback, request, realm, and
authentication contracts remain unchanged. Rested time SHALL NOT change
eligibility, provenance, evidence, report ordering, or no-retry behavior, nor
authorize live verification.

#### Scenario: Reporting a rested eligible character

- **WHEN** an otherwise eligible online browser or desktop character reaches
  a report-producing transition under rested progression
- **THEN** the runtime attempts the existing profile-compatible report after
  committing the exact transition snapshot and accounting, with serialized
  delivery and no retry, without a new rested-evidence gate

#### Scenario: Retaining existing operation evidence

- **WHEN** existing operation evidence passes its unchanged non-pacing
  requirements and only production rested timing differs
- **THEN** that timing difference alone does not invalidate the operation's
  readiness or mark its character local-only

#### Scenario: Preserving an existing gate

- **WHEN** an offline-originated, local-only, unsupported, or otherwise gated
  character has rested time
- **THEN** its existing reporting restrictions remain in force

#### Scenario: Completing implementation without live verification

- **WHEN** rested progression satisfies its local requirements and focused
  synthetic checks
- **THEN** implementation can finish without sending live verification
  requests, issuing new passing evidence, or claiming normal leaderboard
  classification

#### Scenario: Exercising development conformance tooling

- **WHEN** an existing development conformance runner advances its separately
  authorized experimental character
- **THEN** it retains the existing unaccelerated active-time contract and
  does not earn or consume a managed rested bank

#### Scenario: Documenting rested readiness without live claims

- **WHEN** rested progression is presented in inspection or documentation
- **THEN** existing operation readiness is distinguished from unverified
  rested-timeline acceptance and leaderboard classification, and dashboard
  views omit the long acceptance notice
