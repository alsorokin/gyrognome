# Design

## Context

The shared desktop simulator, exact realm contracts, passkey-only transport,
operation-scoped evidence, and independent UI gates are implemented. Four
Pemptus records are pinned: unadapted manual/motto and placeholder-only level/act.
The completed source/synthetic study covers the exact indexed quest placeholder,
including all five unsigned request shapes. The guild lifecycle completed, but
its response fingerprints were not retained. Historical approvals, execution,
and cleanup remain recorded in `tasks.md`.

The remaining problem is readiness policy, not missing transport or a second
simulator. Strict original-path lookup splits the four supported operations
between paths; mandatory fingerprints block guilds. This revision replaces that
partial-delivery policy with usable support on three narrowly defined paths.

## Goals / Non-Goals

**Goals:** All five operations for otherwise eligible Pemptus imports with no
adaptations, spelling-only adaptation, or exact quest-placeholder-only adaptation;
consistent inspection/runtime/UI behavior; one-shot confirmed guild actions;
targeted delivery without another lengthy live campaign.

**Non-Goals:** Exhaustive Delphi/anti-cheat certification, arbitrary adaptation
equivalence, combined adaptations, new destinations or credentials, retries,
reconnecting local-only history, or relabeling observations. No new live
experiment is authorized by this revision.

## Decisions

### 1. Reuse the exact native contracts and existing implementation

| Realm | Saved endpoint | HTTPS destination | Credentials/header |
|---|---|---|---|
| Spoltog | `http://progressquest.com/spoltog.php?` | `https://progressquest.com/spoltog.php` | Complete account/password; Basic |
| Pemptus | `http://progressquest.com/pemptus.php?` | `https://progressquest.com/pemptus.php` | Positive passkey, empty account/password; no Authorization |

Keep shared signing, request constructors, fixed destinations, redirect refusal,
bounded transport, credential-safe errors, callback ordering, and online-action
serialization. Do not change the importer, simulator, or real save. Browser and
Spoltog behavior, including the original Spoltog v2 payload/digest, stay intact.

### 2. Add explicit supported-path equivalence, not generic fallback

For Pemptus only, approve one canonical-behavior family: `[]`,
`[load-spelling-patch]`, and `[legacy-quest-placeholder]`. Placeholder admission
still requires the recognized exact marker, valid pinned monster-table index,
and otherwise supported source. Unknown tasks/captions, legacy prologue, invalid
indexes, and combined adaptations remain excluded.

Reuse existing actual level, act, manual, and motto records across this family.
This is an explicit readiness policy justified by the completed source/synthetic
study and native results, not a claim that each original path received its own
live campaign. Original provenance remains authoritative after marker resolution.

Prefer an exact-path record, retaining the established spelling equivalence.
Only absence of a matching record may invoke the explicitly approved equivalent
path for the same Pemptus contract and operation. An exact record that is stale,
invalid, inconclusive, or duplicated blocks that operation rather than falling
through. An ambiguous or invalid equivalent selection likewise fails closed.
Never share across realms or operations.

Validate the selected record's original source/implementation identity, payload,
registered adaptations, integrity, freshness, acceptance, classification, and
cleanup as before. Equivalent input coverage does not change its declared
observation path. Do not alter fixture bytes, digests, v3 emission rules, or make
the offline study a production evidence envelope.

**Alternative rejected:** Duplicating and relabeling the four fixtures would
fabricate observations; another per-path campaign adds delay without addressing
the already established narrow equivalence.

### 3. Use an explicit Pemptus public-confirmation guild strategy

Represent guild verification as a real strategy: existing fingerprint rules for
Spoltog and public membership confirmation for supported Pemptus imports. Guild
readiness uses the exact Pemptus source/contract, supported family, completed
lifecycle, and shared public parser; it does not require a fabricated guild
record or dummy fingerprints. Other import, credential, encoding, and local-only
checks still apply.

Reuse the existing native guild constructor and send at most one mutation.
After the attempt, perform one public membership observation through the
existing bounded reader/transport. No polling loop, mutation retry, or queued
replay is needed. A delivery error may still be reconciled because the server
could have applied the request; an unconfirmed result remains non-success.

Match the exact character in the Pemptus realm and reconcile guild names
ASCII-case-insensitively, persisting observed canonical spelling. An empty leave
requires authoritative no-guild state. Preserve the existing narrow handling of
an omitted empty trailing Guild cell only when the table explicitly declares it
and every preceding cell aligns unambiguously. Missing headers, malformed cells,
duplicates, conflicting rows, or observation failures remain unknown.

Persist membership only on confirmation. Otherwise keep the previous membership
and report a safe categorized non-success outcome, including invalid designations
that do not produce the requested membership. A matching pre-existing membership
confirms desired state, not uniquely attributable request consumption.

**Alternative rejected:** HTTP 200 or an empty body is not confirmation.
Requiring lost fingerprints prevents an otherwise usable action; inventing them
would misrepresent the retained evidence.

### 4. Apply shared readiness everywhere and preserve timeline history

Save inspection, managed eligibility, worker/manual targets, profile actions,
and dashboard use the same supported-path policy. Existing unadvanced imports,
including Izot-equivalent records, gain all five gates without reimport solely
because their import predates this revision.

Automatic eligibility still determines local-only advancement. Supported
records with current valid level/act coverage do not fork solely because their
original path differs from a record's observed path. Missing valid automatic
coverage still atomically creates permanent local-only provenance on advancement.
Already local-only timelines never reconnect; fresh official-client import
remains their recovery path.

Keep motto-before-delivery persistence, accepted-only guild persistence, and
profile/worker locking. UI controls retain existing requested-operation guards.
Uniform eligible Pemptus imports show no obsolete path-mismatch/fork warning;
genuine partial or blocked states remain visible. Rendering sends no requests.

### 5. Finish with focused checks and honest documentation

Exercise the three-path/five-operation matrix, invalid selection boundaries,
one-shot guild confirmation, local-only permanence, and shared surface behavior.
Use existing transport/public-reader doubles and source-derived synthetic states.
Verify exact observable readiness and state transitions, not just enum plumbing.
Run directly affected Rust/browser tests and formatting; broader suites only if
the change or failures warrant them. Do not repeat completed live cases.

Update README and compatibility docs to describe usable supported-path coverage,
excluded combinations, public-confirmation uncertainty, and the distinction
between readiness and exhaustive live conformance. Retain historical evidence
gaps without presenting them as current feature-completion blockers.

## Risks / Trade-offs

- Narrow source/synthetic equivalence plus existing observations provides less
  assurance than live validation of every path/subcase. Document that limitation;
  do not claim Delphi runtime equivalence or anti-cheat certification.
- Public state may lag or markup may change. A bounded failed observation leaves
  membership unchanged and reports uncertainty; it never triggers a retry.
- Shared selection changes could affect Spoltog or hide broken evidence. Scope
  equivalence to Pemptus and test exact-record failure precedence.
- Historical local-only characters remain excluded. Do not erase provenance to
  make the new support appear universal.

## Migration Plan

No database migration or credential replacement is required. Implement shared
readiness and the guild strategy, wire existing consumers, update documentation,
and complete the focused practical acceptance checks in section 9 of `tasks.md`.
Previously completed work remains historical, not proof the new scope is done.

Rollback disables the new policy/strategy without changing managed history or
evidence. If automatic readiness is later lost, existing permanent local-only
advancement semantics and visible warning still apply.
