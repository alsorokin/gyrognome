# Tasks

## 1. Pane Model and Shortcuts

- [x] 1.1 Remove Status from the collapsible pane/visibility model, assign F1-F6 to Activity, Progress, Equipment, Details, Adventure, and Journal, and verify command and pane-header unit tests cover the new mapping and ignored F7 input
- [x] 1.2 Update collapsed-pane behavior tests to cover only the six collapsible panes and verify `cargo test dashboard::tests` passes

## 2. Full Dashboard Rendering

- [x] 2.1 Render the full-layout Journal title as `Journal - {bestplot}`, remove the separate Adventure plot content row, retain inventory/spells/conditional quest behavior, and verify rendering tests cover `Journal - Act VIII` without an Adventure `Plot:` row
- [x] 2.2 Replace the separate full-layout Status and Keys rows with one horizontal bottom row containing equal-width Keys and Status panes, keep Status permanently expanded without a hotkey, and verify rendering tests cover pane order, visibility, and wrapping at 90 and 120 columns
- [x] 2.3 Preserve the compact dashboard's stacked Status/Keys and plot presentation, and verify compact rendering assertions still pass
- [x] 2.4 Reduce normal full-layout Keys and Status panes to one inner-content row, compact their standard text to fit at 90 columns, preserve taller safety warnings, and verify focused dashboard tests pass
- [x] 2.5 Add spacing between full-layout shortcut groups, render the `q`, `b`, `m`, `g`, and `s` shortcut keys in bold, and verify styled-line and dashboard rendering tests pass
- [x] 2.6 Remove pipe separators from the full-layout Keys line, retain one space between shortcut/action groups and bold shortcut keys, and verify focused rendering tests pass
- [x] 2.7 Restore ` | ` separators in the full-layout Keys line, widen Keys just enough to fit the complete line at 90 columns, give Status the remaining width, and verify layout and styled rendering tests pass
- [x] 2.8 Align Keys and Status to the main one-third/two-thirds column ratio, allow the complete Keys line to wrap across two content rows, restore the full Status label, and verify 90- and 120-column rendering tests pass
- [x] 2.9 Make the full-layout bottom row use one content row when the complete Keys content fits and expand to two only when it wraps, while preserving aligned panes and warning height; verify narrow and wide layout tests pass
- [x] 2.10 Insert an empty Adventure row between Spells and the conditional `Current quest:` line when Journal is collapsed, and verify both collapsed and expanded Journal line layouts
- [x] 2.11 Make the compact Keys pane use the shared styled shortcut line, allocate one content row when it fits and two only when it wraps, preserve confirmation-warning sizing, and verify compact-width tests pass

## 3. Documentation and Release Metadata

- [x] 3.1 Update README full-layout documentation for the fixed bottom Status pane and F1-F6 assignments, and verify the documented order matches rendered pane headers
- [x] 3.2 Bump the project version from `1.1.0` to `1.1.1` in `Cargo.toml`, the root package entry in `Cargo.lock`, `package.json`, and the root package fields in `package-lock.json`; verify `cargo metadata --no-deps --format-version 1` and a Node JSON read report `1.1.1`

## 4. Validation

- [x] 4.1 Run `cargo fmt --check`, the full `cargo test` suite, and `npm test`, and verify all dashboard, Rust, and conformance tests pass
