# Tasks

## 1. Follow newly observed Adventure updates

- [x] 1.1 Reuse completion-interval inventory/spell diffs in `DashboardState::replace_character` to select a one-shot Adventure reveal target in display order; add focused tests covering both `refresh` and `read_state`, duplicate-name entries, multiple changes, removals, and unchanged refreshes.
- [x] 1.2 Map that entry to its actual wrapped row range when rendering Adventure and apply the smallest clamped offset change; verify `TestBackend` output reveals off-screen inventory items and spells, retains fully visible targets, and shows the start of over-height entries while preserving inline formatting, whitespace handling, and Unicode cell widths.

## 2. Respect manual reading

- [x] 2.1 Add a session-local 30-second monotonic pause to the shared Adventure manual-scroll path for arrow keys, page keys, and mouse wheel, cancelling any unrendered reveal; verify injected-time tests cover repeated scrolling, boundary attempts, suppression before expiration, eligibility at expiration, no replay on expiration, and new updates after expiration.
- [x] 2.2 Discard opportunities observed while Adventure is collapsed or compact, and keep automatic revealing independent of focus and other pane offsets; verify focused tests preserve Journal and compact scrolling and show no replay on expansion or layout changes.

## 3. Check the integrated behavior

- [x] 3.1 Run `cargo test dashboard::tests` and `cargo fmt --check`; resolve regressions in update highlighting, wrapped rendering, manual scrolling, and pane layout before marking the change complete. All 99 dashboard tests pass. Repository-wide formatting reports pre-existing differences in `src/desktop_export.rs`, `src/export.rs`, `src/lib.rs`, and `src/state.rs`; per user approval, leave these files untouched. `rustfmt --edition 2024 --check src/dashboard.rs` passes.
