# Tasks

## 1. Capture recent task updates

- [x] 1.1 Extend the credential-safe dashboard character snapshot with stats and introduce display-only recent-update keys for fixed stats/equipment and named inventory/spell entries; verify no raw save document or online credentials are added to dashboard snapshots.
- [x] 1.2 Update dashboard refresh state to compare the prior and successful refreshed snapshot only when completed-task count advances, replace prior markers at that boundary, and retain markers otherwise; verify focused state-transition tests cover preservation, replacement, and failed refreshes.
- [x] 1.3 Implement logical-identity comparisons for stat values, equipment slots, inventory quantities, and spell ranks, including additions, removals, reordering, and duplicate names; verify focused unit tests assert only current changed entries are selected.

## 2. Render highlighted character values

- [x] 2.1 Add reusable styled line/span rendering for stats, inventory, spells, and equipment using the dashboard's established terminal selection treatment; verify Ratatui buffer tests distinguish highlighted and unhighlighted values.
- [x] 2.2 Render stats and recent task update indicators in the full layout without adding panes or changing existing pane allocation; verify existing full-layout geometry and shortcut tests continue to pass.
- [x] 2.3 Render stats and the same indicators in compact layout while preserving wrapping and the compact-width threshold; verify compact rendering contains styled changed and unstyled unchanged values.

## 3. Validate dashboard behavior

- [x] 3.1 Extend dashboard tests for a task-completion refresh that updates each supported value category, an ordinary refresh that retains the indicator, and a later completion that clears prior markers; verify with `cargo test dashboard`.
- [x] 3.2 Run the project’s relevant Rust test suite and formatting checks; verify the dashboard remains credential-safe and no simulation, persistence, or lifecycle behavior regresses.

## 4. Refine stats placement

- [x] 4.1 Move stats from the Adventure and compact content areas into the responsive dashboard header, aligning them right beside identity when they fit and placing them on a second header row otherwise; verify Ratatui layout tests cover both placements and stat highlight styling.

## 5. Refine Adventure presentation

- [x] 5.1 Show Adventure's current quest only while Journal is collapsed and separate inventory from spells with one empty line; verify focused rendering tests cover both Journal visibility states and list spacing.
