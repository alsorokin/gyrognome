# Proposal

## Why

Dashboard letter hotkeys currently match lowercase characters only, so Caps Lock makes them stop working. Accepting either ASCII case restores the existing controls without changing their actions.

## What Changes

- Make dashboard letter shortcuts and character-selector letter navigation/cancellation case-insensitive.
- Recognize Ctrl+C with either case of `c` in the dashboard, selector, and profile editors.
- Preserve the exact case of text typed into motto and guild editors, existing confirmation/eligibility checks, and non-letter controls.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `terminal-dashboard`: Require letter shortcuts to work regardless of Caps Lock while preserving profile input case.

## Impact

Changes are limited to keyboard dispatch and focused tests in `src/dashboard.rs`, plus a short clarification in README dashboard controls. No new dependencies, storage changes, CLI options, or runtime/reporting behavior changes are needed.
