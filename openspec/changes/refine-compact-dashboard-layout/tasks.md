# Tasks

## 1. Compact rendering

- [x] 1.1 Update `render_compact` to show canonical `Current quest:` instead of joined history and insert blank lines before/after equipment and after spells. Add row-aware rendering assertions covering distinct current/completed quests, wrapped section content, and empty equipment; verify history remains absent throughout the scrollable content and the separators occupy their intended rows.
- [x] 1.2 Remove task percentage from `progress_text`, reserve a dedicated row, and render the existing `ProgressGauge` there using the passed predicted percentage and wrapped-prefix measurement described in `design.md`. Verify Ratatui buffer symbols and colors match full-mode fill at 0, 75, and 100 percent, the label is centered, and the non-task summary remains unchanged.

## 2. Scrolling, regression checks, and documentation

- [x] 2.1 Add compact tests for wrapped activity/percentage text, partial eligibility content, and scrolling the gauge into/out of view; adjust overflow fixtures to use retained content if necessary. Verify no border/content overwrite, keyboard and mouse scrolling still work, resizing clamps offsets, and predicted rendering leaves persisted state unchanged.
- [x] 2.2 Update the compact-layout description in `README.md` to describe current-quest-only display, the dedicated task bar, and section spacing; verify it agrees with the delta spec and preserves existing full-mode documentation.
- [x] 2.3 Run `cargo test dashboard::tests` and `cargo fmt --check`; verify the dashboard tests cover both compact and full layouts and resolve regressions attributable to this change without reverting the pre-existing staged edits.

## 3. Task bar ordering follow-up

- [x] 3.1 Move the compact task bar immediately above the other percentage summary, preserving section spacing and scrolling. Update documentation and change artifacts to match; verify row-order assertions, wrapped-content scrolling tests, and `cargo fmt --check` pass.
