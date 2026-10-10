# Proposal

## Why

Small dashboard changes currently replace requirements that also govern unrelated
selection, quest presentation, collapse controls, and layout rules. Split the
two clearest bundles so future deltas can target cohesive contracts.

## What Changes

- Split Managed character dashboard into dashboard opening/state overview,
  character selection/recency, and Journal/Adventure quest presentation.
- Split Collapsible dashboard panes into collapse controls/hotkeys, full-layout
  space allocation, Keys/Status arrangement, and compact-layout fallback.
- Preserve every existing clause and all 25 scenarios without changing behavior.
- Retain the capability path and all unaffected requirement names; record
  migration mappings for the two retired bundle names.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `terminal-dashboard`: Replace two bundled requirements with seven smaller
  requirements. This is structural spec refactoring, not new product behavior.

## Impact

Only OpenSpec organization changes; no source, tests, dependencies, runtime
behavior, or user-facing documentation changes are required. Existing ID/profile
presentation conflicts remain explicitly outside this refactor. Pemptus
requirements and historical archives are untouched.
