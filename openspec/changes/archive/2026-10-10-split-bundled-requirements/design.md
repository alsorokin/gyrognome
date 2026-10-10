# Design

## Context

See proposal.md for motivation. OpenSpec replaces named requirements as whole
blocks, so changing file layout or introducing subheadings would not reduce the
delta replacement boundary. There were no active changes before this refactor.

## Goals / Non-Goals

**Goals:** Make the replacement boundary match a cohesive contract and preserve
the complete existing contract across the resulting requirements.

**Non-Goals:** No implementation changes, capability moves, rewritten historical
deltas, repository-wide reorganization, or one-requirement-per-scenario policy.

## Decisions

Remove the two original requirement bundles with explicit Reason and Migration
fields, and add all replacement requirements together. The installed OpenSpec
validator rejects a MODIFIED block that omits any original scenario, even when
that scenario moves into another requirement in the same delta. Removal plus
addition expresses the structural split without duplicating scenarios or
weakening validation. No product feature is removed.

| Original requirement | Resulting requirement | Existing scenarios |
| --- | --- | --- |
| Browser ruleset fidelity | Bundled browser ruleset fidelity | Rule lookup and level transition (2) |
| Browser ruleset fidelity | Canonical Specialty selection | Ranked spell, tie, no learned spells (3) |
| Browser ruleset fidelity | Canonical Prime Stat selection | Current stat and tie (2) |
| Controlled offline advancement | Runtime active-time advancement | Active runtime, restart, simulation error, delay, invalid duration (5) |
| Controlled offline advancement | Browser runtime scheduling and persistence | Completion deadline, interval cap, tick equivalence, full task (4) |
| Controlled offline advancement | Desktop runtime callback pacing | Fixed pacing, missed callbacks, rested pacing, rested completion, rest exhaustion (5) |
| Controlled offline advancement | Desktop runtime commit boundaries | Completion, reporting, partial progress, full bar, graceful stop, abnormal exit, bounded checkpoint (7) |

Keep scenario names and bodies verbatim. Keep normative sentences verbatim
apart from whitespace needed to reflow extracted paragraphs. This makes loss
or weakening detectable without executing product code.

Browser scheduling and persistence stay together because each successful
scheduled update is a commit. Desktop pacing and persistence separate because
many callbacks deliberately do not commit. Rested pacing stays with normal
desktop pacing: they are modes of the same clock contract, not independent
features. Splitting every scenario would obscure these relationships.

## Risks / Trade-offs

- Dropped clauses or scenarios: compare the original blocks with the combined
  replacements before synchronization, ignoring only whitespace and headings.
- Future changes targeting old bundles: record the replacement names in the
  removal migrations and inspect the current main spec when preparing later
  deltas. Unaffected names remain unchanged.
- Large bundles remain elsewhere: this is a bounded first pass, not a claim
  that all oversized requirements have been addressed.

## Migration Plan

There is no runtime migration. Verify the delta preserves all clauses and
scenarios, then synchronize both capability deltas into the main specs as one
refactor. Keep archived changes unchanged. A rollback would restore the two
original bundled requirement blocks without touching product code.
