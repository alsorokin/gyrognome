# Tasks

## 1. Update full-layout quest presentation

- [x] 1.1 In `src/dashboard.rs`, append the non-empty canonical current quest to the Journal title only when collapsed, preserving the plot caption; verify rendered title assertions for collapsed, expanded, empty-quest, and refreshed states, including when Adventure is also collapsed.
- [x] 1.2 Remove the current-quest branch and visibility parameter from `adventure_lines` and update all callers; replace the old fallback test with assertions that Adventure contains only inventory, its separator, and spells in either Journal state.

## 2. Verify rendering and nearby behavior

- [x] 2.1 Add rendered-buffer coverage for long collapsed titles at the minimum full-layout width, including a wide-character caption; verify F6 remains visible, the title stays inside the header, and Journal retains its two-row height, constraining the title width locally if needed.
- [x] 2.2 Run `cargo test dashboard::tests` and `cargo fmt --check`; verify the new title tests and existing expanded Journal, compact current-quest, pane-collapse, and Adventure auto-scroll tests pass.

Validation: all 101 dashboard tests pass. `src/dashboard.rs` passes its
formatting check. Repository-wide `cargo fmt --check` reports pre-existing
formatting differences in `src/desktop_export.rs`, `src/export.rs`,
`src/lib.rs`, and `src/state.rs`; those unrelated files remain unchanged.
