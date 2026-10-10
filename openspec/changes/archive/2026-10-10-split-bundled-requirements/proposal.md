# Proposal

## Why

Some requirements combine independently changeable contracts, forcing small
changes to repeat unrelated clauses and scenarios in delta specs. Split two
clear bundles first so future changes have smaller replacement boundaries.

## What Changes

- Split Browser ruleset fidelity into ruleset fidelity, canonical Specialty
  selection, and canonical Prime Stat selection.
- Split Controlled offline advancement into shared runtime advancement,
  browser scheduling, desktop callback pacing, and desktop commit contracts.
- Retain every existing obligation and scenario without changing behavior.
- Keep capability paths and unaffected requirement names unchanged; explicitly
  map the two retired bundle names to their replacements.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `deterministic-simulation`: Reorganize Browser ruleset fidelity into three
  independently editable requirements; no behavioral change.
- `local-character-runtime`: Reorganize Controlled offline advancement into
  four independently editable requirements; no behavioral change.

These are structural spec modifications, not new product capabilities.

## Impact

Only OpenSpec requirement organization changes. No source, tests, APIs,
dependencies, runtime behavior, or compatibility guarantees change. Other
large requirements remain outside this first pass, and archived changes remain
historical records rather than being rewritten.
