## 1. Dashboard interaction state

- [x] 1.1 Add fixed F1-through-F7 commands and session-local expanded/collapsed state for Activity, Progress, Equipment, Details, Status, Adventure, and Journal; verify focused command/state tests show each key toggles only its assigned pane and preserves lifecycle controls.
- [x] 1.2 Pass pane visibility state through full dashboard rendering while keeping compact rendering unchanged; verify dashboard rendering tests cover both width modes and representative collapse combinations.

## 2. Pane rendering and layout

- [x] 2.1 Introduce consistent right-aligned F-key hints in the headers of all collapsible full-layout panes; verify rendered output contains the correct key hint for each pane.
- [x] 2.2 Update full-layout constraints so collapsed panes become header-only or relinquish their content space to visible panes, cap expanded Journal at one quarter of the usable right-column height, fix expanded Details at five rows, and fix expanded Progress at seven rows; verify rendering succeeds when individual panes and multiple panes are collapsed and asserts the Journal, Details, and Progress height constraints.
- [x] 2.3 Change Adventure to display existing adventure details and one plain `Current quest:` line, and add Journal with the bold current-quest value only first followed by completed quests from most recent to oldest; verify rendering tests assert the distinct Adventure label, unlabeled Journal value, reverse ordering, and styling for a long quest history.

## 3. Validation

- [x] 3.1 Run the targeted dashboard test suite and resolve any regressions in refresh, lifecycle confirmation, compact view, quest rendering, F-key interactions, or Journal, Details, and Progress height allocation.
- [x] 3.2 Run the repository's standard Rust formatting and test commands to verify the completed dashboard behavior integrates cleanly.
