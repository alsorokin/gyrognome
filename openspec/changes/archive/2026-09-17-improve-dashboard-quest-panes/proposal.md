## Why

The Adventure pane combines the current quest with an unbounded completed-quest history, making the currently actionable objective difficult to scan. Dashboard operators also need to reclaim terminal space without losing access to individual character details.

## What Changes

- Simplify the Adventure pane to show its existing adventure data and one non-bold `Current quest:` value instead of the completed-quest list.
- Add a Journal pane that lists completed quests from most recent to oldest and presents the current quest in bold.
- Make the Activity, Progress, Equipment, Details, Status, Adventure, and Journal panes independently collapsible with F1 through F7 hotkeys, and display each pane's hotkey in its right-aligned header.
- Preserve the compact dashboard view while adapting the full dashboard layout to the new collapsible panes.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `terminal-dashboard`: Revise dashboard quest presentation and add keyboard-controlled collapsible dashboard panes with visible hotkey hints.

## Impact

- Affects terminal dashboard rendering, layout allocation, and keyboard command/state handling in `src/dashboard.rs`.
- Requires dashboard rendering and input-handling test coverage updates.
- Does not change character persistence, runtime lifecycle operations, public CLI commands, or dependencies.
