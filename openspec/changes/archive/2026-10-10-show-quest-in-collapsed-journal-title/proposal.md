# Proposal

## Why

Keep the current quest visible in the collapsed Journal's title instead of mixing quest information into Adventure. This keeps quest context attached to Journal and leaves Adventure focused on inventory and spells.

## What Changes

- Append the canonical current quest to the collapsed Journal title: `Journal - <plot caption> - <current quest>`.
- Remove the full-layout Adventure current-quest row and its extra separator, regardless of Journal visibility.
- Preserve the expanded Journal title, bold current quest, and reverse-chronological completed history.
- Preserve compact layout quest presentation and existing pane controls.
- Omit the quest suffix when the canonical current quest is empty; retain existing header clipping for long titles.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `terminal-dashboard`: Replace the collapsed-Journal Adventure quest fallback with a current-quest suffix on the Journal title.

## Impact

The change is confined to full-layout rendering and nearby tests in `src/dashboard.rs`, plus the existing `terminal-dashboard` behavior contract. It supersedes that spec's explicit requirement to show `Current quest:` in Adventure when Journal is collapsed. No dependencies, simulation, persisted state, online operations, or compact-layout behavior change.
