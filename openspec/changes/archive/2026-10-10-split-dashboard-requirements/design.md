# Design

## Context

See proposal.md for motivation. Named requirements are the replacement boundary
for OpenSpec deltas; additional subheadings or separate files would not solve
the problem. README dashboard controls describe existing selection, pane toggles,
and compact fallback; this refactor does not introduce new UI behavior.

## Goals / Non-Goals

**Goals:** Separate independently changeable contracts while preserving the
complete existing statements and scenario bodies.

**Non-Goals:** No product implementation, capability moves, historical delta
rewrites, Pemptus changes, or reconciliation of ID/profile presentation conflicts.

## Decisions

Use REMOVED requirements with explicit migration mappings and ADDED replacements.
The installed validator rejects scenario removal from a MODIFIED block even
when those scenarios are moved into other blocks in the same delta. Reusing
the old name for a shortened MODIFIED bundle would therefore fail validation;
keeping every scenario there would preserve the bloat.

| Original bundle | Replacement | Existing scenarios |
| --- | --- | --- |
| Managed character dashboard | Dashboard opening and state overview | Registered and unknown character (2) |
| Managed character dashboard | Dashboard character selection and recency | Selection, recency/activity, cancellation, empty store (4) |
| Managed character dashboard | Full-layout Journal and Adventure presentation | Plot, expanded/collapsed quest, expansion, refresh, empty/long title (7) |
| Collapsible dashboard panes | Dashboard pane collapse controls | Collapse, restore, Status visibility, hotkey discovery (4) |
| Collapsible dashboard panes | Full-layout pane space allocation | Journal, Activity, Details, collapsed Details, Progress, Equipment heights (6) |
| Collapsible dashboard panes | Dashboard Keys and Status arrangement | Bottom-row arrangement and wrapping (1) |
| Collapsible dashboard panes | Compact dashboard fallback | Compact view and Keys wrapping (1) |

Keep the existing sentences verbatim except for whitespace reflow; regroup them
by responsibility, including moving the unknown-character error sentence to
dashboard opening. Keep all scenario names and bodies verbatim and exactly once.
Compare sentence inventories per original bundle, not just total counts:
sentence order changes as obligations move into their corresponding blocks.

Journal title/content behavior remains together with Adventure's omission of
duplicate quest/plot content because those describe the same presentation
decision. Pane sizes remain one allocation contract rather than separate
requirements for every numeric height. Keys wrapping and confirmation height
belong together, independently of pane collapse behavior.

## Risks / Trade-offs

- Lost or weakened obligations: compare whitespace-normalized sentence
  inventories and complete scenario inventories against the original bundles.
- Existing conflicting ID/profile requirements: leave Visible rested
  progression, Concise profile and timing details, and Visible compatibility
  and eligibility unchanged. Resolving their disagreement requires a separate
  behavioral decision, not an incidental rewrite.
- Future references to retired names: record mappings in the removal migrations
  and use current main requirements when drafting later changes.

## Migration Plan

Verify the mappings, then replace each original bundle in place with its mapped
requirements, preserving all unaffected blocks and the capability Purpose.
The main spec will have 21 requirements, up from 16. No runtime migration or
product test run is needed; validate the main specs and compare the exact diff.
Rollback restores the two original bundles without touching source code.
