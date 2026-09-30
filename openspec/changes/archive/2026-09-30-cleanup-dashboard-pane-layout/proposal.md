# Proposal

## Why

The full dashboard dedicates scarce vertical space to plot text and places runtime status in a separate collapsible row above the keyboard help. Consolidating plot context into the Journal title and pairing the fixed Status and Keys panes will make the layout denser and keep essential runtime feedback continuously visible.

## What Changes

- Remove the plot content row from the full-layout Adventure pane.
- Include the canonical current plot caption in the Journal pane title using the compact form `Journal - <plot caption>`, for example `Journal - Act VIII`.
- Move Status to the bottom row beside Keys, with Keys on the left and Status on the right.
- Make Status non-collapsible and remove its F-key shortcut.
- Reassign F1 through F6 in visual pane order to Activity, Progress, Equipment, Details, Adventure, and Journal.
- Keep compact character and Status presentation intact while making the compact Keys pane height responsive to whether its shortcut line wraps.
- Bump the project patch version from `1.1.0` to `1.1.1` across Rust and Node package metadata and lockfiles.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `terminal-dashboard`: Change the full-dashboard Adventure content, Journal title, bottom pane layout, Status visibility behavior, and collapsible-pane hotkey assignments.

## Impact

- Affects dashboard layout, pane rendering, keyboard command mapping, and dashboard rendering/layout tests in `src/dashboard.rs`.
- Updates the full-layout shortcut documentation in `README.md`.
- Updates release versions in `Cargo.toml`, `Cargo.lock`, `package.json`, and `package-lock.json`.
- Does not change persisted character data, simulation behavior, service lifecycle semantics, compact-dashboard content, or external APIs.
