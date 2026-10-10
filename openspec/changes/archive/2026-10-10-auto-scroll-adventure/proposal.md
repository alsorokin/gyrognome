# Proposal

## Why

Automatically bring newly added or changed inventory items and spells into view in Adventure so updates are not hidden outside its viewport. Preserve manual reading by pausing this behavior for 30 seconds after the user scrolls Adventure.

## What Changes

- Reveal the first added or changed inventory item or spell in display order when a persisted task-completion update is observed and Adventure auto-scroll is enabled.
- Pause auto-scroll for 30 seconds after each manual Adventure scroll, using monotonic time. Further manual scrolling restarts the pause.
- Discard auto-scroll opportunities observed during the pause; expiration alone does not move the viewport or replay an older update.
- Keep the existing inline inventory/spell layout, update highlighting, keyboard focus, and manual scrolling. Only Adventure's full-layout offset may move automatically.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `terminal-dashboard`: Add Adventure update-following behavior with a 30-second manual-scroll pause, without changing Journal or compact scrolling.

## Impact

The change is confined to dashboard state/update detection, Adventure wrapped-content targeting and scroll-event handling in `src/dashboard.rs`, and focused dashboard tests. No new dependencies, persistent settings, simulation changes, network requests, or changes to Journal are planned.
