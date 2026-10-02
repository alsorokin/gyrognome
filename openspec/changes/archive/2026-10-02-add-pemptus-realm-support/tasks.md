# Tasks

Sections 1-8 and their execution notes retain completed historical work under
the previous partial-support policy. Their old gate assertions are historical,
not the current completion contract. Section 9 carries the approved usable-support
revision in proposal/design/specs; this change is unfinished until it is complete.

During apply, reuse the existing implementation and actual pinned records.
Do not relabel observations, erase provenance, reconnect local-only records,
repeat completed live cases, or reconstruct deleted private artifacts. No new
live experiment is authorized or required; any future experiment still needs
separate explicit scope approval. The missing guild fingerprint audit remains
non-conclusive even though the new public-confirmation strategy removes it as
a production readiness prerequisite.

## 1. Shared realm contracts and independent evidence

- [x] 1.1 Add a shared exact desktop realm/endpoint/credential-mode contract registry for Spoltog and Pemptus; verify unit coverage rejects unknown realms, endpoint aliases/userinfo, wrong credential modes, incomplete credentials, and unsupported encoding without transport.
- [x] 1.2 Add separately parsed operation-scoped evidence records, explicit encoding/contract identity, independent integrity pins, and conditional guild coverage validation in `desktop_evidence`; verify synthetic records cover matching and mismatched scopes, expired/tampered/inconclusive records, completed cleanup, and non-guild records without fingerprints.
- [x] 1.3 Preserve the strict Spoltog v2 reader and original digest/scope through the new selector; verify its existing operations remain valid without granting Pemptus coverage and invalid Pemptus guild evidence does not invalidate independently valid non-guild records.
- [x] 1.4 Replace the global all-operation eligibility dependency with requested-contract/operation selection; verify a matrix of all five operations for both realms, partial coverage, absent production Pemptus records, supported spelling-only equivalence, and rejection of substantive/combined adaptations.

## 2. Native transport and realm-specific profile classification

- [x] 2.1 Make verified transport authentication explicit for Basic versus passkey-only contracts; verify captured constructed requests omit Authorization entirely for Pemptus, retain the existing Spoltog header, and do not leak authentication between sequential realm requests.
- [x] 2.2 Preserve fixed HTTPS delivery, redirect/downgrade refusal, bounded failures, redaction, and single-attempt behavior; verify transport tests cover hostile saved destinations, wrong credentials, timeout/response limits, and no retry.
- [x] 2.3 Select guild normalization/fingerprints and exact-name public reconciliation from matching realm evidence; support Pemptus's single omitted empty trailing guild cell only with the matching table's explicit header and unambiguous complete preceding row. Share the existing ASCII-case-insensitive guild-name reconciliation between production and development while preserving exact character identity and canonical public guild spelling. Verify case-only join/change differences, leave, absent headers, unsupported spans, duplicate/extra/malformed cells, cross-realm rejection, shared development/production interpretation, and unchanged strict Spoltog parsing/response vectors.
- [x] 2.4 Keep the archived Pemptus diagnostic development-only and non-enabling while reusing shared contracts where appropriate; verify its synthetic tests still distinguish delivery/state agreement from independently established acceptance.

## 3. Inspection, managed runtime, reporting, and dashboard

- [x] 3.1 Wire shared contract/evidence decisions through save inspection, managed runtime presentation, worker targets, and manual report delivery; verify equivalent synthetic imports receive identical inspection/runtime operation decisions and gated requests never reach the adapter.
- [x] 3.2 Wire Pemptus motto set/clear and guild join/change/leave into the existing profile-action paths; verify motto-before-delivery persistence, accepted-only guild persistence, cancellation/invalid-input safety, and serialization with worker reports using synthetic adapters.
- [x] 3.3 Preserve automatic level/act callback behavior and durable local-only provenance; verify persisted-before-delivery ordering, no catch-up/replay, atomic first advancement with either automatic gate missing, restart/evidence-update permanence, untouched-import eligibility, and no fork caused solely by missing manual/motto/guild evidence.
- [x] 3.4 Add mixed-operation dashboard Details presentation without new controls; verify manual-only and motto-without-guild states, uniform summaries, requested-operation guards, local-only recovery messaging, and no network/state mutation during rendering or refresh.

## 4. Separately scoped development conformance tooling

- [x] 4.1 Extend the feature-gated runner/CLI with exact realm, stage, operation, attempt/observation bounds, existing-disposable/preparatory-leave options, explicitly approved preparation-only read-only reconciliation, and a separate explicit operator-accepted join continuation scope. Verify missing approval, insufficient seven/eight-attempt bounds, unauthorized empty baselines/preparation/recovery, changed scopes, wrong join intent/checkpoint shape, other uncertain primary operations, and progression recovery fail before mutation; preserve safe Spoltog invocations.
- [x] 4.2 Implement immediate-stage manual consumption attribution using an approved distinct motto marker, separate motto set/clear, two valid guild join/change designations, invalid designation, and final leave observations. Support the explicitly approved older level-1 disposable exception with exact public/saved baseline agreement, an approved empty control motto, and one confirmed initial guild leave when needed; verify preparation cannot satisfy final-leave coverage and unchanged HTTP-success rows cannot establish manual acceptance.
- [x] 4.3 Support independently scoped real-time level/act stages using shared native import/simulation/request/transport; verify synthetic callback tests preserve six-to-eight-hour active-time validation, deadlines, no acceleration, and inconclusive outcomes for missing accepted transitions.
- [x] 4.4 Preserve intent-before-mutation checkpoints and no uncertain replay. Retain preparation-only recovery and add the separately approved five-attempt/three-observation operator-accepted join exception with unchanged source/scope, exact reconstructed join intent, recorded preparation recovery, no callback progress, and fresh 60-second normal report-field/linked-guild reconciliation. Preserve original intents/count and observations, record operator authorization/timing separately without a response fingerprint, skip only the accepted join, and continue only guild change/rejection/final leave under the original eight-attempt cap. Verify zero reconciliation mutations, no completed-case replay, intent/scope/checkpoint mismatch and expiration refusal, restart safety, private-only cleanup, original preservation, and checkpoint removal after reusable sources.
- [x] 4.5 Emit sanitized operation records compatible with the validator and shared guild fingerprints; exclude preparation/recovery/operator acceptance from operation proof and never invent a missing response fingerprint or promote an expired observation. After successful final no-guild/private cleanup, emit only independently complete records and explicitly report guild exclusion when operator-accepted join lacks its original response fingerprint. Verify production selection/classification replay, privacy, fresh primary acceptance and final-leave requirements, independent manual/motto promotion, and rejection of abnormal, stale, incomplete, ambiguous, preparation-only, recovery-only, or operator-acceptance-only enabling results.

## 5. Closed-gate documentation and integration readiness

- [x] 5.1 Update README and desktop compatibility documentation for realm contracts, independent/partial eligibility, local-only consequences, older-disposable immediate scope, declared trailing-cell parsing, shared case-insensitive guild matching, preparation-only reconciliation, and the separately approved non-enabling operator-accepted join continuation without replay. Explain independently conclusive record emission, missing-fingerprint guild exclusion, original preservation, and unchanged progression prerequisites; verify examples match help and no premature production-support claim is made.
- [x] 5.2 Run `cargo fmt --check`, `cargo test --features desktop-live-conformance`, and `npm test` after targeted continuation/casing tests; do not repeat completed live cases. Verify browser and existing Spoltog regressions pass, all Pemptus production gates remain closed without records, and default builds do not expose development live tooling. Classify pre-existing lint failures separately without unrelated fixes.

## 6. Approved Pemptus immediate-operation evidence

- [x] 6.1 Verify the recorded immediate scope and freshly approved operator-accepted join continuation against the unchanged source and existing five-attempt/three-primary-observation checkpoint with completed preparation recovery. Confirm the sole pending join intent, no competing join request, and no callback progress. Preserve no official-client mutations, approved guilds/markers, exclusive stopped reporters, private-only cleanup, and the original eight-attempt total cap; revalidate all scope conditions before continuation.
- [x] 6.2 Verify and document the retained record of completed guild lifecycle, approved private cleanup, and original preservation; explicitly retain the missing response-fingerprint audit as non-conclusive/non-enabling. Verify guild remains gated for both supported Pemptus import paths and no operator acceptance or other-path record substitutes for missing evidence. Do not recover fabricated fingerprints, repeat completed live checks, or label the original audit passing.
- [x] 6.3 Confirm final immediate-stage no-guild state and approved private-copy/checkpoint cleanup while preserving original save/backup, then validate and pin only independently conclusive operation records. Explicitly exclude guild production coverage without the original join response fingerprint; verify operator acceptance, preparation-only, incomplete cleanup, and unchanged-row-only results cannot produce enabling records.

Original task 6.2 before the approved partial-delivery revision:

> Under task 6.1's approved bounds, reconcile the operator-accepted linked guild read-only without replay, retain completed manual consumption/motto set-clear and accepted join, then attempt only guild change to QoD, invalid-designation rejection, and final leave. Require fresh 60-second observations, normal classification, and unambiguous available fingerprints for those unattempted cases under the original eight-attempt cap. Do not rerun completed cases or turn operator acceptance into timely join evidence; failed reconciliation or missing acceptance leaves affected subcases blocked.

Execution paused on 2026-10-01: approved read-only preparation recovery succeeded
without replay. Manual consumption and motto set/clear were observed; guild join
was attempted but not conclusively recorded within its bound. The ledger has
five total attempts (including the original preparation), three primary
observations, and separate recovery timing. A separately approved read-only
diagnosis found normal report-field agreement and linked membership displayed
as `BEERguild`, differing from the approved request spelling `BEERGuild`.
**Guild-join subcase: complete by explicit operator acceptance.** The operator
confirmed that no competing guild-join request was made and accepted the
observed linked membership as the result of the recorded native request.
The runner's case-sensitive comparison caused the unrecorded acceptance;
the existing production reconciliation already compares guild names
case-insensitively. The operator explicitly requested no repeat validation.
This acceptance does not fabricate a within-deadline observation, response
fingerprint, or enabling record, and did not itself authorize mutation replay
or primary checkpoint recovery. The original checkpoint/timing facts are retained.
At that pause, guild change, rejection, and final leave were unattempted;
private copies/checkpoint were retained and no records had been promoted.
Subsequently, the operator explicitly approved the narrow design/conformance-spec/
tasks revision and implementation for read-only reconciliation of this accepted
join, then only the three unattempted guild cases and private-only cleanup.
All reporters were confirmed stopped. Each new observation is bounded to
60 seconds; the total cap remains eight. No completed live case is to be
repeated, no join fingerprint is to be fabricated, and guild production
evidence remains closed under this exception. The seven reopened implementation/
readiness tasks concern the new continuation path, not repeat live validation.

Continuation completed on 2026-10-01: read-only reconciliation accepted the
normal linked guild with matching report fields, then only change to QoD,
invalid-designation rejection, and final leave were attempted. Fresh bounded
observations succeeded within the original eight-attempt total cap; no completed
live case was repeated and no progression occurred. Final normal report-field/
no-guild checks passed. Original save/backup hashes remained unchanged; approved
private save/backup and checkpoint were removed. The sanitized output contains
only independently conclusive manual-brag and motto envelopes, now bundled
with their original, separately pinned integrity values. Guild remains gated.
The original join fingerprint is absent, and subsequent guild fingerprints
were not retained in the emitted records after private checkpoint cleanup.
Task 6.2's lifecycle subcases are complete, but its available-fingerprint
audit cannot be established from retained output; the overall checkbox remains
unfinished rather than inventing coverage or repeating the completed run.

## 7. Approved Pemptus progression evidence

- [x] 7.0.1 Trace and pin the official 6.4.4 new-character, load/save, quest-selection, and marker-use semantics for an exact `fQuest` placeholder with a valid saved monster-table index. Establish the candidate canonical replacement and its scope from integrity-verified source rather than assuming cosmetic equivalence or writer identity from layout; explicitly exclude unsupported `fTask`, invalid indexes/captions, legacy prologue, and other substantive/combined adaptations. Use the approved private source only for offline shape diagnosis; retain no real save or credential in proof artifacts.
- [x] 7.0.2 Build credential-free synthetic placeholder/canonical pairs and differential checks for canonical game state after the relevant load normalization, RNG consumption, callback/report ordering, initial/subsequent quest transitions, and all five unsigned operation request shapes. Cover valid/invalid indexes, unknown captions, unsupported task forms, legacy prologue, spelling-only combinations, and other adaptations; preserve visible provenance and verify every current production/live gate remains unchanged. Synthetic clocks and requests must not count as live acceptance, active-time coverage, or operation evidence.
- [x] 7.0.3 Record and review a scoped pass/fail/inconclusive proof result. Keep failure or uncertainty blocked; even a passing result must not change eligibility, relabel bundled records, or authorize requests. Obtain a separately approved planning revision before any adaptation/evidence policy change and a separately verified identity/source/scope handoff before live execution. Do not repeat completed immediate cases or silently widen the existing progression approval.
- [x] 7.0.4 Implement explicit `--allow-quest-placeholder` admission only for Pemptus progression requesting both automatic operations with exactly `[legacy-quest-placeholder]` in validated original-source provenance, the exact marker, and a valid pinned-table index. Bind the flag/source/state/index/adaptation path into checkpoint scope; preserve ordinary task resolution and original save/history. Verify missing/wrong flag, unadapted flag use, another realm/stage, unknown marker, unsupported task, invalid index, and combined adaptations fail before network; changed scope and uncertain primary requests cannot replay.
- [x] 7.0.5 Emit new v3 automatic records from actual original-source provenance and accept the singleton placeholder path only for Pemptus automatic level/act. Make independently pinned registry selection import-path-aware across eligibility, inspection, runtime, worker/manual/profile reporting, preserving existing unadapted/spelling equivalence. Verify exact scope selection, distinct-path coexistence, duplicate matching rejection, no absent/stale/invalid/inconclusive cross-path fallback, and no widening/relabeling of existing records or inheritance by manual/motto/guild/combined paths.
- [x] 7.0.6 Update directly related documentation/CLI examples and run synthetic positive/negative source/checkpoint/evidence/inspection/runtime/local-only tests plus `cargo fmt --check`, `cargo test --features desktop-live-conformance`, `cargo check --bins --no-default-features`, and `npm test`. Verify visible provenance survives marker resolution; synthetic evidence cannot reconnect local-only records; existing Pemptus manual/motto, Spoltog v2 scope/digest, browser behavior, and production gates without new records remain unchanged. Do not send live requests or repeat completed immediate checks.
- [x] 7.1 Freshly confirm the exact newly created disposable official-client Pemptus identity/source, placeholder-only admission when applicable, automatic operations, at most 32 native attempts/28,800 active seconds/60-second acceptance observations, exclusive stopped-reporter handoff, original preservation, and private-only cleanup; verify the recorded source/scope offline before running. Do not infer live authorization from the policy revision or exhausted immediate approvals. No additional official-client preparation is authorized; changed source or uncertain state requires guidance. This task remains blocked until the revised live scope is explicitly approved and its handoff validated.
- [x] 7.2 Under task 7.1's approved scope, execute native real-time progression and observe accepted level and act transitions plus normal classification; verify no acceleration or replay occurred and leave missing/ambiguous/expired coverage unfinished rather than extending the scope without approval.
- [x] 7.3 Confirm progression-stage cleanup and validate/pin conclusive automatic-operation records; verify each record has matching identities, native transition acceptance, integrity, freshness, normal classification, and completed cleanup before promotion.

Progression approval received on 2026-10-01: one newly created disposable
official-client Pemptus character; official preparation limited to creation,
one control-motto set, and one baseline manual brag; native automatic level/act
only, at most 32 mutation attempts and 28,800 active seconds, with 60-second
acceptance observations. The operator confirmed stopped competing reporters,
original preservation, and successful-run private-only cleanup. The approved
control motto is `Gyrognome progression control`.
Private copies were prepared outside the checkout, but initial network-free
validation found an empty saved motto instead of the approved control. Inspection
of that initial source confirmed a supported unadapted Pemptus import.
The operator subsequently confirmed the updated official save and stopped-client
handoff. Refreshed private copies were validated without network activity, but
the updated save fails import with `desktop current task form is unsupported`.
Task 7.1 remains unfinished until a supported source/control handoff is validated;
no live progression requests or callbacks have been started, and original saves
were not modified by the runner.
Approved offline diagnosis found the literal current-task caption `fTask` during
the initial prologue, at level 1 with one plot row and zero quest rows. The save
is structurally readable, but that caption is outside the supported task codes;
no safe canonical normalization has been established. No importer bypass or
real-save edit was made. A supported official-client handoff after the prologue
requires separate approval for any additional official reports before native
progression can begin.
The operator then approved a handoff correction bounded to 30 official-client
active seconds, at most one startup act report and one additional baseline
manual brag, and confirmed the resaved level-1 source and stopped reporters.
The refreshed save imports and its motto matches the approved control, but
it records `legacy-quest-placeholder`, which the approved unadapted progression
scope rejects. The placeholder cannot be silently stripped or treated as
matching the existing evidence scope. Task 7.1 remains blocked, with zero native
requests/callbacks, pending a supported unadapted handoff; client version must
not be inferred from the recognized save layout.
Read-only verification of the operator-provided official executable confirmed
the exact pinned 6.4.4 executable hash; the executable was not launched.
The adaptation blocker is therefore not resolved by changing the asserted
client version. The pinned, integrity-verified source also confirms that a new
character's quest marker begins empty. No equivalence exemption or additional
official-client operation has been authorized to address the carried-forward
placeholder.
The operator subsequently approved planning-only revisions for an offline
placeholder-equivalence study in design, conformance requirements, and these
three new prerequisites. This approval does not establish equivalence, complete
those tasks, relax adaptation gates, or authorize further live preparation.

Offline study completed on 2026-10-01. The independently hash-verified pinned
Main.pas source initializes the quest caption empty, restores/serializes component
properties without marker normalization, and selects a quest monster using only
non-empty marker presence and the saved table index. The exact placeholder and
indexed canonical monster row therefore have matching source-level selection
semantics within the declared narrow scope; their saved caption strings are not
identical and no Delphi component-stream runtime equivalence is claimed.

Reproducible synthetic tests compare monster selection before normalization for
every valid table index and 64 RNG seeds, including a differing-index negative
control. They compare initial and subsequent quest transitions over 64 seeds,
three transitions per pair, complete callback state/RNG continuation, ordered
level/act snapshots, and unsigned manual/level/act/motto-set/clear/guild shapes.
Canonical game state converges through the existing native task-completion
resolution, not an importer change or provenance erasure. Invalid indexes,
unknown captions, unsupported fTask, legacy prologue, and spelling/placeholder
combinations retain their boundaries and existing gates.

Reviewed result: pass within this exact source-and-synthetic scope only, recorded
in the credential-free `desktop-quest-placeholder-study.json` fixture. The
evidence selector rejects that study for all five operations. Changes are test
and documentation only; no production or live gate was changed, bundled operation
records were not relabeled, no real save was edited, and no live request was sent.
The full feature-gated Rust suite and production-only binary check pass.
Task 7.1 remains blocked by the current adaptation policy. Any exemption or
broader evidence scope needs a separately approved planning revision, followed
by a freshly verified source/identity/exclusive handoff before live execution.

Subsequently approved planning revision: implement a strictly declared
placeholder-only Pemptus progression path and actual-adaptation automatic
evidence, as specified in 7.0.4-7.0.6. This refines the earlier policy blocker
but does not implement admission, produce live evidence, or complete 7.1.
No new official-client operation or completed immediate-case replay is
authorized. Existing unadapted manual/motto evidence remains restricted to its
original path; new placeholder automatic records cannot enable unadapted
imports or manual/profile actions. Live scope and handoff must be reconfirmed
after the new implementation prerequisites pass.

Placeholder policy implementation completed on 2026-10-01. Explicit admission
requires the sole validated placeholder adaptation and the pinned-table index;
the revised checkpoint binds the flag and original source path without changing
ordinary marker resolution or the historical unflagged scope. V3 automatic
records retain original-source adaptations after resolution. Path-aware registry
selection rejects duplicates, relabeling, and absent/stale/invalid/inconclusive
cross-path fallback. Synthetic runtime tests establish automatic-only eligibility,
inspection/runtime agreement, preserved visible provenance, and permanent
local-only exclusion when either automatic record is missing.

The full feature-gated Rust suite, formatting check, production-only binary
check, 67 browser tests, and strict OpenSpec validation pass. Existing manual/
motto records and Spoltog evidence remain unchanged; no new live records are
bundled. No real source was edited, no completed immediate case was repeated,
and no live request or progression callback was started. Task 7.1 remains
blocked until fresh live-scope approval and offline handoff verification.

Revised live scope approved on 2026-10-01: the operator explicitly authorized
starting the session-attached placeholder-only automatic experiment after
offline verification, confirmed the unchanged corrected disposable source
and control motto, and reconfirmed exclusive stopped reporters for the run.
Both originals and private copies match the recorded corrected-handoff hashes.
The revised native `--validate-only` invocation passes without network activity
or checkpoint creation. Bounds remain 32 native attempts, 28,800 active seconds,
and 60-second acceptance observations; original preservation/private-only
cleanup and no additional official preparation or immediate-case replay apply.
Task 7.1 is complete; live acceptance, cleanup, and promotion remain unfinished.

The approved native progression process is now running session-attached.
Startup verification confirms an advancing private checkpoint within scope
and unchanged originals; no automatic acceptance was recorded at that check.
Task 7.2 remains in progress. The shell identifier is retained only in the
session-private approval manifest; no enabling record has been promoted.

The progression process subsequently exited inconclusive after 22,429.7 active
seconds and 13 native attempts. Its retained ledger contains 12 accepted level
observations and one accepted act observation, with no pending callback.
Read-only diagnosis identifies the evidence-emission duplicate-case guard:
it rejects repeated level cases even though native progression emits successive
level reports before the first act transition. No retry or restart was made.
The runner retained private sources/checkpoint and emitted only a non-enabling
diagnostic; originals remain hash-identical. Cleanup and evidence promotion
remain incomplete pending guidance on a no-replay packaging correction.

The operator explicitly approved fixing packaging and finalizing the existing
run without replay, and reconfirmed stopped reporters. The corrected guard
allows successive automatic cases only with distinct native intents while
retaining duplicate immediate-case, repeated-intent, pending-callback, and
unobserved-request rejection. Regression tests prove completed progression
performs no callback, observation, or delivery and leaves its ledger unchanged.
The 13-intent completed ledger and unchanged source/scope were verified offline
before finalization; no new progression or mutation was performed.

Fresh read-only final-state checks and approved cleanup succeeded. Private
save/backup/checkpoint were removed; both original hashes remain unchanged.
Validated v3 automatic records declare the actual singleton placeholder path
and normal classification/completed cleanup. Independently pinned level and act
digests are respectively `15ca8cb23f43f382392d4587640b8d53fdbd9819c1d42bbe8ab303ea119b353a`
and `c258dbfe95d5d16fa97613389c8ec9278cfe90128eed6de715abbf72f4d25ee0`.
Level coverage is observed October 1, 2026, valid through October 1, 2027; act
coverage is observed October 2, 2026, valid through October 2, 2027.
The pre-cleanup private checkpoint SHA-256 is retained in the session manifest
for the completed-ledger audit, not as a replacement for operation evidence.

## 8. Evidence promotion and partial-support completion

- [x] 8.1 Bundle only validated Pemptus records from completed approved stages into the operation registry; verify partial coverage opens only its matching gates, record removal/staleness recloses them, and no local-only managed timeline becomes eligible.
- [x] 8.1.1 After the separately approved placeholder progression stage and cleanup pass, validate and independently pin only conclusive automatic records declaring the actual original-source adaptation path. Verify matching-path positive eligibility, unadapted/manual/motto/guild/combined-path negatives, path-specific removal/staleness, and permanent local-only exclusion. Do not relabel the existing manual/motto records or promote the offline study as operation evidence.
- [x] 8.2 Verify/update README and desktop compatibility documentation for the validated partial matrix: unadapted/spelling-only manual/motto and singleton-placeholder automatic level/act, with guild, unadapted automatic reporting, placeholder manual/profile actions, and combined paths gated. Document local-only consequences and the retained guild evidence gap; defer full parity and missing coverage without combining paths, authorizing live work, or erasing history.
- [x] 8.3 Run final targeted eligibility/transport/runtime/profile/dashboard/evidence regressions plus `cargo fmt --check`, `cargo test --features desktop-live-conformance`, `cargo check --bins --no-default-features`, and `npm test`; verify every delta scenario against the declared partial scope, including matching-path positives, unsupported/absent/stale/invalid-path negatives, evidence privacy/integrity, permanent local-only exclusion, browser/Spoltog preservation, documented guild audit gap, and approved cleanup/original preservation. Complete only this partial-support change, with no additional live requests, replay, or full-parity claim.

Placeholder automatic promotion is complete. Independent real-record tests
verify both pins, exact-path positive coverage, unadapted/spelling/manual/motto/
guild/combined-path negatives, path-specific removal and staleness, and isolation
from existing manual/motto evidence. Production runtime regressions verify
inspection agreement, persisted-before-delivery automatic reporting, preserved
placeholder provenance after ordinary resolution, and permanent local-only
exclusion when records become available after an offline fork.

Post-promotion formatting, full feature-gated Rust regressions (349 library
tests plus binary/integration suites), production-only binary checks, 67 browser
tests, and strict OpenSpec validation pass. The offline study still cannot
enable any operation without separately installed records. No existing manual/
motto or Spoltog record was rewritten. Full-support completion remains
unfinished: no one Pemptus import path covers all five operations, guild
fingerprint coverage is incomplete, and adapted automatic/unadapted profile
records cannot be combined to claim parity.

Partial-delivery completion scope approved on 2026-10-02. The operator chose
to finish validated partial support with guild and unsupported paths gated,
then explicitly approved revisions to proposal, design, conformance specs,
and these tasks. This changes the current completion criteria; it does not
make the original guild fingerprint audit conclusive or achieve full parity.
The original task 6.2 and its missing evidence remain recorded above.
Tasks 6.2, 8.2, and 8.3 now verify documented gap closure, partial-support
documentation, and final partial-scope regressions; they remain unchecked
until applied and verified. No implementation/evidence policy was weakened,
no new live scope was approved, and no completed case may be repeated.

Revised tasks 6.2 and 8.2 verified on 2026-10-02. Retained sanitized stage outputs
contain only manual/motto and placeholder level/act records, all with confirmed
cleanup and normal classification; neither contains guild evidence. Both private
stage directories contain only their sanitized evidence, without reusable inputs
or checkpoints. Immediate and progression original hashes match the recorded
preserved sources. Targeted desktop regressions pass, including operator-accepted
join exclusion and both supported paths' closed guild gates.

README and compatibility documentation now explicitly distinguish delivered
partial support from deferred parity, retain the missing guild audit, and explain
the different local-only consequences of unadapted versus placeholder imports.
Dashboard regression coverage includes both delivered partial combinations.
Completing the revised documentation task does not make the original fingerprint
audit conclusive, broaden a record, or authorize any further live validation.

Final partial-scope sign-off completed on 2026-10-02. All 63 delta scenarios
were checked against implementation, regression coverage, and retained evidence:

| Capability | Scenarios | Verification scope |
|---|---|---|
| Leaderboard conformance | 40 | Independent record validation, partial matrix/gap documentation, non-enabling offline proof, placeholder admission/provenance, exact-path selection, staged authorization/acceptance, no replay, cleanup, and archived-probe rejection |
| Online character profile | 6 | Conditional realm-specific motto/guild construction and persistence, matching synthetic guild classification, unknown/rejected-result preservation, serialization, and actual closed guild gates |
| Opt-in leaderboard reporting | 12 | Real bundled path-specific gates, inspection/runtime agreement, persisted automatic snapshots, mismatched contracts/combined paths, permanent local-only forks, and preserved browser/Spoltog behavior |
| Terminal dashboard | 5 | Partial/uniform presentation, requested-operation controls, both delivered partial combinations, local-only recovery, and read-only rendering |

Synthetic conditional guild coverage remains testing only, not production proof.
The original missing fingerprint audit remains non-conclusive. Real matching
records enable only unadapted/spelling-only manual/motto and placeholder-only
level/act; absent, stale, invalid, duplicate, or other-path records fail closed.
Retained sanitized stage outputs and original hashes confirm approved cleanup
and original preservation without reconstructing deleted evidence.

Final validation passes: 153 targeted desktop tests; formatting; all 349 library
tests plus binary/integration/fixture-safety suites under
`cargo test --features desktop-live-conformance`; production-only binaries under
`cargo check --bins --no-default-features`; 67 browser tests; and strict OpenSpec
validation. No new live request or repeated experiment was performed. At that
sign-off, all tasks were complete only for the then-approved partial-support
scope, and full usable coverage was outside that scope.

## 9. Complete usable Pemptus support

The operator subsequently approved restoring practical support on unadapted,
spelling-only, and exact indexed quest-placeholder-only paths. This explicitly
replaces the earlier no-cross-path/partial-delivery policy with narrow readiness
equivalence and publicly confirmed guild actions. Existing actual records and
their observation metadata remain unchanged. The source/synthetic study remains
an equivalence justification, not a live evidence envelope or Delphi certification.

- [x] 9.1 Implement explicit Pemptus supported-family readiness for level, act, manual, and motto using existing pinned records without changing observed paths/digests. Prefer exact coverage; allow approved equivalent coverage only when matching coverage is absent. Verify the three-path operation matrix, preserved original provenance, exact-record stale/invalid/inconclusive/duplicate blocking, ambiguous/invalid equivalent selection, and rejection of wrong realms or unsupported/combined paths with targeted eligibility/evidence tests.
- [x] 9.2 Add an explicit Pemptus public-confirmation guild strategy alongside unchanged Spoltog fingerprints, reusing native request construction and the bounded realm-aware reader. Verify at most one mutation and one observation, join/change/leave with canonical case reconciliation, invalid designation and failed/ambiguous/malformed observations preserving prior membership, uncertain-delivery reconciliation without retry, and no fabricated guild record/fingerprints using existing adapters and profile/reporting tests.
- [x] 9.3 Wire the shared readiness through inspection, managed runtime, worker/manual/profile targets, CLI listing, and dashboard; update README and compatibility docs for usable supported-path coverage and lower conformance assurance. Verify all five gates for supported imports, no obsolete path-mismatch/fork warning, preserved motto/worker ordering and read-only rendering, and unchanged local-only recovery messaging with directly affected runtime/dashboard tests.
- [x] 9.4 Verify practical end-to-end readiness with existing pinned records and synthetic imports for all three paths, including a previously imported unadvanced Izot-equivalent record becoming all-five eligible without reimport and advancing without a path-mismatch local-only fork. Verify already-local-only timelines stay excluded, missing valid automatic coverage still forks atomically, original adaptation history survives marker resolution, and existing Spoltog/browser behavior remains intact. Run focused affected tests plus formatting, escalating only for demonstrated cross-cutting risk or failures; verify evidence bytes/digests are unchanged. Do not repeat a live campaign or treat the historical 35-task sign-off as completion of this revision.

Usable-support implementation completed on 2026-10-02. The three-path regression
uses actual bundled records and existing managed identities, verifies the exact
five eligible CLI status lines and absent fork notice, exercises manual/motto/
guild actions, and completes an automatic callback without reimport or local-only
fork. Placeholder provenance survives resolution and restart. Separate negatives
retain permanent local-only history and missing-automatic atomic forks.

Pemptus guild verification is explicitly public-membership based, with one native
attempt and one bounded observation; case reconciliation, leave, uncertain
delivery, invalid designation, and failed observations preserve the intended
persistence/no-retry behavior. Historical emitted guild records still validate
their actual fingerprints, but production does not classify Pemptus responses
from them. No missing fingerprint or live observation was manufactured.

Focused desktop (153), runtime (56), dashboard (69), and reporting (24) test
selections pass, as do formatting, production binary checks, strict OpenSpec
validation, and patch whitespace checks. Actual evidence fixtures/pins remain
unchanged. Documentation describes supported-family readiness and its lower
conformance assurance. No live request, repeated experiment, private-artifact
reconstruction, or modification to player saves/installed binaries occurred.
