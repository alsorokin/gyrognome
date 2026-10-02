# Spec Delta

## ADDED Requirements

### Requirement: Independent classic operation evidence

Classic production eligibility SHALL validate evidence for the requested
realm, endpoint, credential mode, supported import path, encoding, and operation
without requiring passing coverage for unrelated operations. Every enabling
record SHALL retain the existing source/implementation identity, integrity,
freshness, normal-classification, and completed-cleanup requirements.

Non-guild evidence SHALL NOT require guild-response fingerprints. New
operation-scoped fingerprint-based guild evidence SHALL require complete matching
join/change/leave and rejection coverage using the shared versioned normalization
contract. Missing, stale, malformed, or mismatched guild-only evidence SHALL NOT
invalidate independently valid non-guild evidence. Supported Pemptus guild
actions SHALL instead use the explicit public-confirmation readiness contract;
the absence of such fingerprints SHALL NOT block that strategy.

Previously bundled Spoltog evidence SHALL remain usable within its original
scope only when its evidence-relevant behavior remains unchanged. Evidence
SHALL NOT be copied or relabeled to cover a different realm or authentication
contract.

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

### Requirement: Usable Pemptus delivery under explicit equivalence

Otherwise eligible Pemptus imports on the unadapted, load-spelling-only, and
exact quest-placeholder-only paths SHALL support automatic level, automatic act,
manual brag, motto set/clear, and guild join/change/leave. The exact placeholder
SHALL require a valid saved pinned-table index and otherwise supported state.
Other or combined adaptations SHALL remain excluded.

The approved readiness policy SHALL reuse existing current valid level/act/
manual/motto observations across that narrow equivalent family, preserving
actual source-path metadata, integrity, freshness, classification, and cleanup.
It SHALL NOT relabel records, manufacture observations, share another realm's
coverage, or treat an offline study alone as live evidence. Supported guild
actions SHALL use one native attempt and bounded public confirmation without
requiring missing historical fingerprints.

Completion SHALL require usable five-operation coverage on every supported path,
consistent inspection/runtime/UI behavior, preserved local-only history, and
directly affected compatibility/privacy regressions. It SHALL NOT require a new
per-path live campaign. Documentation SHALL distinguish readiness from exhaustive
live/Delphi/anti-cheat certification and preserve the non-conclusive guild
fingerprint audit. This revision SHALL NOT authorize a live experiment or replay.

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

### Requirement: Non-enabling offline adaptation studies

An offline study of `legacy-quest-placeholder` equivalence SHALL be limited to
the exact `fQuest` placeholder with a valid saved monster-table index in an
otherwise supported state. It SHALL trace integrity-verified pinned official
source semantics and compare credential-free synthetic placeholder/canonical
pairs for canonical game state after the relevant load normalization, RNG use,
callback/report ordering, and all five unsigned operation request shapes.
Adaptation provenance SHALL remain visible.

Wrong or out-of-range indexes, unknown captions, unsupported task forms,
legacy-prologue state, and other substantive/combined adaptations SHALL NOT
gain eligibility from the study. Real saves, credentials, and signed requests
SHALL NOT enter proof fixtures. Synthetic execution SHALL NOT be represented as
live acceptance or as satisfying real active-time requirements.

A proof result SHALL NOT itself change eligibility, relabel existing evidence,
authorize a request, or waive live acceptance/cleanup requirements. Any
eligibility/evidence policy change SHALL require a separately approved planning
revision; live execution SHALL still require a separately verified matching
identity/source/scope handoff. Failed or inconclusive proof SHALL leave the
existing gates closed.

#### Scenario: Establishing offline equivalence without live authorization

- **WHEN** synthetic comparisons establish the narrowly defined placeholder
  equivalence under verified source semantics
- **THEN** the result is recorded for review without enabling an operation,
  inheriting bundled evidence, or authorizing live execution

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

The development runner MAY accept a freshly approved Pemptus progression source
with exactly `[legacy-quest-placeholder]` recorded as its adaptation path, using
the explicit `--allow-quest-placeholder` scope flag and requesting both automatic
level and act operations. It SHALL validate the original exact `fQuest` marker
and valid saved monster-table index before the existing task-completion
resolution. All other fresh-handoff and realm/authentication requirements SHALL
remain mandatory.

The flag SHALL be refused for an unadapted source, another realm or stage,
missing authorization, an unknown marker caption, unsupported task form, invalid
index, or any combined/other adaptation. The original source/state, saved index,
adaptation path, and flag SHALL be bound into the checkpoint scope. Source
provenance SHALL remain visible and unchanged after runtime marker resolution;
neither real-save edits nor importer/task-normalization changes are authorized.
Changed scope or uncertain primary requests SHALL NOT be resumed by replay.

Successful live coverage MAY emit new automatic-operation v3 records declaring
the actual original-source singleton adaptation path. The validator SHALL accept
that path only for Pemptus automatic level/act operations, with all existing
integrity, source/implementation identity, freshness, attributable acceptance,
normal-classification, and completed-cleanup requirements. The offline study
SHALL remain non-enabling. Existing records SHALL NOT be relabeled or widened.

Record validation SHALL match registered and actual declared source paths.
Readiness selection SHALL prefer an exact contract/operation/path record and
preserve established spelling equivalence. Only absence of a matching record
SHALL permit the explicitly approved Pemptus family equivalence for that same
operation. Stale, invalid, inconclusive, or duplicate matching records SHALL
block that operation without fallback. Ambiguous or invalid equivalent
selections SHALL likewise fail closed. Combined paths SHALL remain excluded.

Planning approval of this policy SHALL NOT authorize live execution. A fresh
matching identity/source/exclusive handoff and explicit bounded live scope SHALL
still be required. No completed immediate case SHALL be repeated. Existing
active-time, no-acceleration, no-retry, observation, and private-only cleanup
requirements SHALL remain unchanged.

#### Scenario: Validating an explicitly scoped placeholder-only source

- **WHEN** a freshly approved Pemptus progression source has the exact indexed
  placeholder as its sole adaptation and the explicit flag is present
- **THEN** offline validation can accept that path while retaining original
  provenance, without sending requests or emitting enabling evidence

#### Scenario: Refusing implicit or broader placeholder admission

- **WHEN** the flag is absent, the stage/realm is wrong, the marker/index is
  invalid, or the source combines placeholder with spelling/prologue adaptation
- **THEN** the runner rejects placeholder admission before network activity

#### Scenario: Refusing a changed placeholder checkpoint scope

- **WHEN** an input source, saved index, adaptation path, or admission flag
  differs from the bound checkpoint scope
- **THEN** continuation fails without replaying a request or rewriting provenance

#### Scenario: Emitting actual adapted automatic evidence

- **WHEN** approved placeholder-only automatic cases pass live acceptance,
  normal classification, and final cleanup
- **THEN** their new records declare `[legacy-quest-placeholder]`, not an empty
  adaptation array inferred from the later canonical callback state

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

### Requirement: Separately authorized Pemptus live conformance

The development conformance procedure SHALL support the exact Pemptus
passkey-only contract through the native desktop import, simulation, request,
and delivery path. Every live stage SHALL require explicit approval of newly
created disposable official-client identities by default, official/native operations,
mutation-attempt bounds, observation bounds, active progression duration where
applicable, exclusive client handoff, and cleanup. Approval of a previous
diagnostic or creation of planning artifacts SHALL NOT authorize another
mutation.

An immediate-stage-only exception MAY use an explicitly approved older
disposable Pemptus official-client character that still passes the fresh
level-1 handoff checks. The exception SHALL require a separately declared scope,
exact normal public/saved baseline and current-guild agreement before mutation,
and exclusive stopped-client handoff. Its approved control motto MAY be empty;
manual and motto-operation markers SHALL remain non-empty ASCII and distinct
from each other and the control baseline. No official-client mutations are
authorized by this exception.

An existing baseline guild SHALL require one explicitly approved preparatory
leave, counted against the same native mutation cap and conclusively observed
before the requested cases. Preparation SHALL NOT satisfy the required final
guild-leave case or confer any other operation's proof. Full immediate coverage
with preparation SHALL require an eight-attempt scope. Interrupted or uncertain
preparation SHALL NOT be replayed or promoted. The exploratory source/backup
SHALL remain untouched; cleanup SHALL remove only approved private copies and
runner-owned state. Progression SHALL retain its separately approved newly
created disposable identity requirement.

Pemptus observations MAY recognize an omitted empty trailing guild cell only
when the matching table explicitly declares that column and the unique
exact-name row has every preceding cell with unambiguous alignment. Missing
headers, unsupported spans, extra columns, duplicate rows, and conflicting
content SHALL remain unknown. Production and development Pemptus readers SHALL
agree; historical Spoltog parsing and response semantics SHALL remain unchanged.

Guild-name reconciliation SHALL use the existing ASCII-case-insensitive
production contract in development as well, retaining the observed public
spelling. Character-row identity SHALL remain an exact-name match.

An explicitly approved fresh read-only window MAY reconcile preparation for an
existing-disposable immediate stage containing exactly one recorded preparatory
leave intent, no conclusive observations, and no callback progress. Recovery
SHALL verify the original source/scope and intent, current normal report-field
agreement, and confirmed no-guild state within its new bound, without another
mutation. The original intent/count SHALL be preserved and recovery recorded
separately with actual timing/authorization, without a fabricated response
fingerprint. Failed or expired recovery SHALL prevent continuation.

Reconciled preparation SHALL NOT become operation evidence or retrospectively
validate an expired observation. The seven requested primary cases SHALL still
require new bounded acceptance, final guild leave, and completed cleanup under
the original eight-attempt total cap. This recovery SHALL NOT apply to unknown
primary operations, progression, changed scopes, or replay of any mutation.

A separate explicitly approved operator-acceptance exception MAY continue the
2026-10-01 immediate checkpoint containing exactly five attempts, three
conclusive manual/motto observations, completed preparation recovery, no
callback progress, and the sole unobserved final guild-join intent. The operator
SHALL confirm no competing join request and accept the linked `BEERguild`
membership resulting from the approved `BEERGuild` request. Continuation SHALL
verify the unchanged source/original scope and exact join intent, then confirm
normal report-field agreement and that linked guild within a freshly approved
60-second read-only window. Mismatch, unknown state, expiration, or another
uncertain primary case SHALL prevent continuation.

Operator acceptance SHALL be recorded separately with actual authorization and
reconciliation timing, preserving the original attempts/count and observations.
It SHALL NOT fabricate a within-deadline join observation or the missing
response fingerprint. Only the unattempted guild change to `QoD`, invalid
designation rejection, and final leave MAY follow, with fresh 60-second primary
observations under the original eight-attempt total cap and stopped-reporter
handoff. Completed preparation, manual, motto, and join cases SHALL NOT be
repeated. This exception SHALL NOT authorize progression or general primary
recovery.

Confirmed final no-guild state and approved private-only cleanup MAY permit
promotion of independently complete manual/motto records. Operator-accepted
join lifecycle completion SHALL NOT provide guild production evidence;
missing join fingerprints SHALL remain an explicit diagnostic audit gap, not
block the separately approved public-confirmation strategy. Original source/backup preservation and all other
production evidence requirements SHALL remain unchanged.

Immediate-operation and automatic-progression stages SHALL be independently
bounded. Immediate coverage SHALL establish manual brag consumption, motto
set/clear, guild join/change/leave, and deliberately rejected guild designation
outcomes. Automatic coverage SHALL exercise native level and act transitions
using actual active time without accelerated simulation or fabricated progress.
Each covered operation SHALL have independently attributable acceptance
evidence and normal leaderboard classification. HTTP success, an empty
response, or an unchanged pre-existing public row alone SHALL be inconclusive.

Attempts SHALL be recorded before mutation so interrupted or uncertain requests
are not replayed. Rejected, ambiguous, missing, expired, or abnormal observations
SHALL leave affected evidence non-passing. Only credential-free, conclusive
operation records with confirmed cleanup SHALL be eligible for production
promotion. Live records SHALL claim only their actually proven operation/path.
Usable-support completion SHALL follow the explicit equivalence/public-confirmation
readiness requirement, not be represented as exhaustive per-path live certification.
The archived acceptance probe SHALL remain diagnostic evidence only.

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
