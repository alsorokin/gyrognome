# Design

## Context

See `proposal.md` for motivation and the four capability deltas for acceptance
requirements.

The importer already accepts the inspected Pemptus component streams as
desktop-6.4.4. The pinned official 6.4.4 source uses shared simulation and report
construction; realm selection changes endpoint/identity and credential
requirements, not progression. Pemptus's published flags omit the account and
password requirements, and `AuthenticateUrl` omits authentication when both
fields are empty.

The current production bottlenecks are concrete:

- `desktop_eligibility` supplies one Spoltog evidence contract to inspection
  and runtime; `desktop_evidence` validates all five operations together.
- The v2 production envelope requires guild fingerprints even for a non-guild
  operation. Its bundled Spoltog payload has a pinned integrity digest.
- `desktop_transport` recognizes only Spoltog and always builds Basic
  authentication. `desktop_profile` likewise selects Spoltog-only guild rules.
- `desktop_live_conformance` assumes Spoltog account/password authentication,
  couples immediate/progression work, and emits an immediate-only result with
  cleanup still incomplete.
- Dashboard actions already consult the requested operation, but the Details
  summary labels the whole character ineligible when any operation is blocked.

The archived Pemptus probe observed HTTP 200, an empty body, normal
classification, and an unchanged agreeing public row. It did not independently
establish report consumption. Its exhausted authorization is not reusable.

## Goals / Non-Goals

**Goals:** One source of truth for realm contracts; independent evidence gates;
shared native production/development construction and authentication; staged
promotion without weakening existing evidence validation.

**Non-Goals:** A new simulator or credential store, arbitrary server discovery,
automatic evidence downloading, production gate overrides, or changing the
meaning of an existing managed timeline. Proposal non-goals also apply.

## Decisions

### 1. Resolve exact static realm contracts before evidence selection

Introduce a small shared desktop contract module, consumed by eligibility,
transport, profile classification, and the development runner. A descriptor
identifies profile, exact realm, exact saved endpoint, fixed HTTPS destination,
credential mode, supported encoding, and evidence-relevant contract identity.

| Realm | Saved endpoint | HTTPS destination | Credentials/header |
|---|---|---|---|
| Spoltog | `http://progressquest.com/spoltog.php?` | `https://progressquest.com/spoltog.php` | Complete account/password; existing Basic contract |
| Pemptus | `http://progressquest.com/pemptus.php?` | `https://progressquest.com/pemptus.php` | Positive passkey, both account/password empty; no Authorization |

Use the existing credential-mode and operation types. Do not infer a realm from
an endpoint suffix or silently downgrade partially supplied credentials.
Unknown realms, mismatched endpoints, and inappropriate credential modes fail
closed with safe existing-category errors where applicable. Recognition alone
does not confer production eligibility.

**Alternative rejected:** Copying the diagnostic's Pemptus constants into each
consumer would leave inspection, workers, and profile actions vulnerable to
contract drift. Fetching realm mappings dynamically would trust an unevidenced
remote destination.

### 2. Validate and select evidence per operation

Introduce a versioned operation-scoped envelope for new evidence, with a single
covered operation and explicit encoding scope. Preserve the existing source,
implementation, import/adaptation, realm, endpoint, credential-mode, integrity,
freshness, acceptance, classification, and cleanup checks. Guild response data
is required only for the guild operation and covers accepted join/change/leave
and rejected designation outcomes.

Bundle new records separately, with independently pinned digests and registry
entries keyed by contract and operation. A missing or invalid guild record must
not prevent parsing or validating another operation's record. Do not add a
global `all()` validity check, inherit another realm's coverage, or treat
matching HTTPS destinations as matching authentication contracts.

Keep a strict compatibility reader for the current Spoltog v2 envelope. It must
continue to validate the original payload and digest; after validation, expose
only its declared operations under its original scope. Do not rewrite its
payload into allegedly new observations. Preserve its contract identity only
if synthetic request/authentication/classification regressions demonstrate
unchanged evidence-relevant semantics. Any substantive change to those semantics
invalidates affected evidence and requires a separately approved rerun.
Pemptus records use their own contract identity.

Unadapted and load-spelling-only imports can share coverage only through the
already proven canonical equivalence. No new adaptation exemption is added.
Production lookup returns the requested validated record or a safe gating
reason. Guild fingerprints are obtained from that same selected guild record.

**Alternative rejected:** Making fingerprints optional in the existing globally
validated envelope would preserve all-operation coupling and could accidentally
relax the historical guild contract.

#### Offline quest-placeholder equivalence study

The approved progression handoff is currently blocked by the recorded
`legacy-quest-placeholder` adaptation. The supplied official executable matches
the pinned 6.4.4 identity, but that does not prove adaptation equivalence.
Authorize an offline study only, limited to an otherwise supported state with
the exact `fQuest` placeholder and a valid saved monster-table index.

First trace the integrity-verified official source's new-character, load/save,
quest-selection, and marker-use semantics. Establish whether replacing the
placeholder with the corresponding canonical marker preserves behavior; do not
assume that the marker is merely cosmetic or infer save-writer identity from
layout. Explicitly distinguish the unrelated unsupported `fTask` caption.

Then build credential-free synthetic paired states and compare canonical game
state after the relevant load normalization, RNG consumption, callback/report
ordering, and all five unsigned operation request shapes. Exercise initial
quest selection, subsequent quest transitions, and the shared source-derived
simulation paths. Include wrong/out-of-range indexes, unknown captions, legacy
prologue, spelling-only combinations, and other substantive/combined adaptations
as boundaries; they must not acquire eligibility from this study. Keep
adaptation provenance visible. Never place the real source, credentials, or
signed requests in fixtures.

Conclude with a reviewed pass/fail/inconclusive result and explicit scope limits.
This study does not add an adaptation exemption, change the importer, relabel
existing evidence, enable an operation, or authorize live requests. Even a
successful proof requires a separately approved planning revision for any
eligibility/evidence change and a freshly verified live handoff. A failed or
uncertain proof keeps the current strict gates and progression blocker.

### 3. Make authentication an explicit property of the verified target

Replace the opaque always-Basic transport contract with explicit
account/password and passkey-only variants. Verify the credential shape against
the selected contract before building or dispatching a request. Passkey-only
delivery omits Authorization entirely, including an empty Basic header; signing
still uses the imported passkey and shared desktop request constructor.

Retain fixed HTTPS targets, no redirects/downgrades, bounded responses/timeouts,
redacted errors, and no queued or repeated mutations. Authentication must not
leak between sequential realm requests through a reused agent. Test actual
constructed headers using the existing injectable transport boundaries, not
merely an enum value. Production calls still require eligibility; the
development runner's separately confirmed path is not a production bypass.

The diagnostic remains development-only and non-enabling; it may reuse the
shared descriptor without changing its limited outcome semantics.

**Alternative rejected:** A generic saved-URL request or "Basic only when
non-empty" rule would fail to establish which modes are evidenced for a realm.

### 4. Wire every gate and guild observation to the same contract

Update `save` inspection and runtime eligibility to select the same evidence
from profile/import metadata, realm, saved endpoint, credential mode, and
requested operation. Route worker, manual, and profile delivery through the
same contract. Preserve safe output schemas unless an additive field is needed;
do not expose authenticated endpoints or credentials in diagnostics.

Parameterize production guild rules and public fallback by the selected realm.
Reuse the shared fingerprint normalization and exact-name public parsing.
Unknown/ambiguous outcomes never optimistically persist guild membership.
Preserve motto-before-delivery persistence and the online-action lock shared
with worker reports.

Pemptus public markup may omit the empty trailing guild cell while retaining
an explicit Guild column header. The realm-aware reader may interpret exactly
that shape as no guild only when the matching table declares the trailing
column, the unique exact-name row has all preceding cells, and there are no
ambiguous spans, extra columns, or conflicting rows. A missing header or
unidentifiable layout remains unknown. Use this same Pemptus interpretation in
development observations and production fallback; preserve the strict
historical Spoltog reader and response vectors.

Use the existing ASCII-case-insensitive guild-name reconciliation contract in
both production and development, retaining the observed canonical public
spelling as profile state. Character-name matching remains exact. A difference
such as requested `BEERGuild` versus linked `BEERguild` must not cause a
development-only timeout.

Keep existing callback scheduling, ordered level/act snapshots, and desktop
random-state handling unchanged. The local-only decision depends on automatic
level and act eligibility, not whether all five operations pass. An advanced
gated record remains permanently local-only after evidence changes; untouched
imports may become eligible. No credential replacement or database migration
may erase that history.

**Alternative rejected:** Updating only the manual CLI path would leave
inspection, profile edits, automatic reporting, and local-only decisions
inconsistent.

### 5. Present partial availability without redesigning dashboard controls

Keep existing brag/editor key bindings and their per-operation guards. Preserve
the concise summary for uniform desktop outcomes and browser presentation.
For mixed availability, show a partial summary and safe status/reason lines for
the five operations in Details. Local-only status and fresh-import recovery
remain authoritative. Rendering/navigation/refresh do not make network calls.

Update documentation that currently describes overall eligibility as an
all-five requirement. Explain separately that running without either automatic
gate causes a permanent local-only fork even if manual/profile actions pass.

**Alternative rejected:** Keeping the existing overall "ineligible" label would
misrepresent independently eligible actions; a new interaction model is
unnecessary.

### 6. Extend the native runner with independently approved stages

Add explicit realm/stage/operation selection and attempt bounds to the
feature-gated runner, preserving existing Spoltog invocation behavior where
safe. A Pemptus stage requires explicit matching realm approval and save
metadata; it never guesses authorization from the input file.

Before either stage, obtain fresh approval covering the official-client control
operations as well as native mutations, disposable identities, approved marker
and guild values, observation deadlines, attempt bounds, exclusive client
handoff, active-time bounds if relevant, and private-artifact cleanup. Use
newly created official-client characters by default and a private experiment
directory outside the checkout; existing player saves and the earlier probe
are not implicitly authorized. No real saves or credentials enter fixtures.

For the Pemptus immediate stage only, an explicitly approved older disposable
official-client character may be used with the existing fresh-state handoff
checks: level 1, supported initial plot/quest counts, unadapted import, and
matching realm/authentication contract. Require an explicit existing-disposable
scope option; do not interpret a copied file or exhausted probe approval as
permission. Progression retains its separate newly created identity requirement.

The approved exception may use the actual saved/public motto as its control
baseline, including an explicitly approved empty value. The manual and motto
markers remain non-empty ASCII, distinct from one another and from that
baseline. Require exact normal public-row and current-guild agreement with the
copied save before any mutation. No fabricated baseline or official-client
mutation is needed in this scope; all reporters remain stopped throughout.

If that baseline has a guild, require a separately approved initial-guild-leave
option and observe its removal before the requested cases. Count this
preparatory request against the same mutation bound: full immediate coverage
then needs eight attempts rather than seven. Checkpoint it as preparation,
not as the required final guild-leave case. Unknown preparation outcomes
prevent continuation unless the narrow read-only recovery below is freshly
approved; they never permit replay or enable evidence. Bind the exception,
baseline, and preparation options into the checkpoint scope.

For an immediate existing-disposable stage with exactly one recorded
preparatory-leave intent, no conclusive observations, and no callback progress,
allow an explicitly approved read-only preparation reconciliation window.
The approved window here is 60 seconds. Verify the original source/scope and
exact preparation intent, require current normal report-field agreement and
authoritatively confirmed no-guild state, and send no mutation during recovery.
Any mismatch, unknown state, or expired recovery window stops continuation.

Preserve the original attempted request and count. Record recovery separately
with its authorization and actual observation time; do not invent a missing
response fingerprint or reinterpret the expired observation as passing
operation evidence. Once preparation is conclusively reconciled, the remaining
seven requested cases may continue under the original eight-attempt total cap.
Each still needs a fresh bounded acceptance observation, and final guild leave
and private-copy cleanup remain mandatory. Recovery does not apply to unknown
primary-operation attempts, progression, or different scopes.

#### Approved operator-accepted join continuation

The immediate stage interrupted on 2026-10-01 has a separately approved,
non-enabling continuation exception. Its unchanged checkpoint contains five
attempts: the original preparatory leave, three conclusively observed manual
and motto cases, and the sole unobserved guild-join intent. Preparation recovery
is already recorded; no callback progress or additional attempt is permitted.
The operator confirmed no competing join request and explicitly accepted the
linked `BEERguild` membership as the result of the recorded `BEERGuild` request.
The casing mismatch is a runner defect, not permission to replay the join.

Require explicit operator-acceptance/continuation approval, unchanged source
and original scope, exact reconstruction of that final join intent, and a fresh
60-second read-only check of normal report-field agreement and the approved
linked guild membership using shared case-insensitive reconciliation. Record
operator authorization and actual reconciliation timing separately, preserving
all original attempts and observations. Do not create a timely automated join
observation or invent the missing response fingerprint.

This separate record may skip only the already accepted join and allow only
the three unattempted cases: change to `QoD`, invalid-designation rejection,
and final leave. Each requires a fresh 60-second bounded observation under the
original eight-attempt total cap and exclusive stopped-reporter handoff.
Manual consumption, motto set/clear, preparation, and join must not be repeated.
A mismatched source/scope/intent, other uncertain primary case, abnormal or
unconfirmed state, or expired reconciliation blocks continuation.

After confirmed no-guild state and approved private-only cleanup, emit and pin
only independently complete, validated operation records. Operator acceptance
is lifecycle completion, not production evidence: without the original join
response fingerprint, guild coverage remains non-enabling. Report that
exclusion explicitly rather than rejecting independently conclusive manual
and motto records or synthesizing missing coverage. Preserve the originals.
This exception does not authorize progression or general primary recovery.

For this exception, copy the approved source and optional matching backup into
the private experiment directory. Preserve the exploratory originals in place.
Only the approved private copies and runner checkpoint are cleanup targets;
retained or uncertain state cannot be promoted.

Immediate and progression stages can use separate disposable characters under
the same contract. An immediate stage can finish cleanup and emit evidence
without waiting for progression. A checkpoint deliberately retained for later
continuation remains non-enabling until cleanup is complete.

Immediate coverage needs stronger observations than the diagnostic:

- Manual brag uses an explicitly approved distinctive motto marker in its
  native snapshot that differs from the public baseline, without inventing
  progress or sending an intervening motto request. Exact public reconciliation
  can then attribute consumption to that manual intent. This proves manual
  coverage only, not the separate motto operation.
- Motto set and clear are separately observed through their own requests.
- Guild join and change use two approved valid existing designations; invalid
  designation and leave are separately observed. Classifier fingerprints must
  satisfy the shared unambiguous acceptance/rejection contract.

Native progression follows the existing real callback contract and six-to-eight
hour active-time bounds, with bounded observations of requested level/act
transitions. No accelerated clock, fabricated save progress, or accumulated
offline reports are allowed. Each accepted transition requires attributable
state observation or a separately validated response contract, plus normal
classification; agreement with an unchanged pre-existing row is insufficient.

Record each intent before mutation. Interrupted or uncertain attempts are not
replayed on restart. Complete operation records include only safe categories,
scope, identity/integrity metadata, timing, and acceptance/classification/cleanup
facts. Reuse native import/construction/delivery and shared guild fingerprinting,
not the JavaScript reference harness as a live sender.

**Alternative rejected:** Repeating the one-shot HTTP-200 probe or allowing
immediate-only evidence with retained private state would not meet acceptance
and cleanup requirements.

### 7. Promote evidence explicitly, never manufacture a passing fixture

Implement all infrastructure with Pemptus production records absent. Synthetic
records may exercise positive paths through test-only injection, not by changing
production eligibility.

After separately approved live stages pass, validate and sanitize each enabling
record, pin its digest and matching identity, and bundle only covered operations.
Partial evidence is a supported gate state, not completion of this change.
Full support and final support documentation require all five operations and
their subcases. A denied approval, missing runtime prerequisite, expired bound,
uncertain acceptance, or failed cleanup leaves the relevant tasks blocked and
the gate closed.

**Alternative rejected:** Shipping a Pemptus descriptor with Spoltog evidence,
the archived probe result, or invented live observations would make false
compatibility and anti-cheat claims.

## Risks / Trade-offs

- [Empty responses or unchanged public state cannot establish acceptance] ->
  Require independently attributable effects or an evidenced response contract;
  otherwise record inconclusive and stop promotion.
- [Public markup or guild responses differ by realm] -> Exercise realm-specific
  parsing and classification with synthetic vectors and approved live checks;
  never fall back to Spoltog assumptions.
- [Cross-cutting refactors change evidenced Spoltog behavior] -> Preserve legacy
  validation and request vectors; invalidate and reapprove any affected contract
  rather than silently retaining its implementation identity.
- [Partial eligibility surprises users who start local progression] -> Show
  operation-level availability and preserve the permanent-fork warning.
- [Progression may not finish within the approved duration] -> Keep the existing
  active-time bound, report incomplete coverage honestly, and request fresh
  approval before a new attempt.
- [Two valid test guilds or the official-client handoff are unavailable] ->
  Treat the corresponding live task as blocked; do not create guilds or reuse
  credentials from another identity to obtain coverage. An older disposable
  character is allowed only under the explicit immediate-stage exception above.
- [Cleanup could remove a user-supplied save] -> Scope removal only to explicitly
  approved disposable files and runner-owned artifacts; never delete the
  exploratory sample or unrelated files.

## Migration Plan

1. Land contract/evidence selection, authentication, surface wiring, tooling,
   documentation, and synthetic regressions with Pemptus gates closed.
2. Revalidate unchanged Spoltog v2 coverage and browser behavior. Existing
   private desktop credential storage already represents empty account/password
   values, so no storage migration is planned.
3. Seek separate approval and execute bounded Pemptus stages; finalize cleanup
   and promote only conclusive operation records.
4. Once all coverage passes, document full imported-character support and finish
   the change. Existing local-only records still require fresh official imports.

Rollback removes or disables Pemptus evidence registry entries; it does not
rewrite managed state or provenance. Losing automatic eligibility during later
advancement invokes the existing local-only behavior, which must remain visible.
