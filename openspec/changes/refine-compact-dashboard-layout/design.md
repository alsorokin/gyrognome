# Design

## Context

See `proposal.md` for motivation. `render_compact` in `src/dashboard.rs`
currently builds one wrapped, scrollable `Paragraph`: activity, task count,
`progress_text`, equipment, inventory, spells, plot, and joined quest history.
The snapshot already exposes canonical `current_quest` for browser and desktop
characters. Full mode uses the custom `ProgressGauge` widget, while compact mode
puts task percentage into `progress_text`.

The meaningful design decision is integrating a gauge widget with an existing
wrapped, scrollable paragraph without changing its viewport or controls.
Existing staged edits to the dashboard, README, and main spec are the baseline;
this change must preserve them.

## Goals / Non-Goals

**Goals:** Keep a single combined compact scroll range, preserve styled update
spans, and reuse the full-layout gauge appearance and prediction input.

**Non-Goals:** New pane types, a permanently pinned task bar, altered full-mode
layout, changed prediction timing, simulation changes, or quest-history
storage changes.

## Decisions

### Reuse the current quest and section helpers

Replace the joined quest list with `Current quest: <current_quest>`, using the
same label as Adventure's current-quest fallback. Keep its existing position
after plot. Continue using `equipment_lines`, `inventory_line`, and
`spells_line` so highlighting is unchanged. Insert blank logical lines before
equipment, after equipment, and after spells. Treat equipment boundaries
consistently even when all slots are empty; retain existing empty-section
content conventions rather than adding new visibility rules.

Reading the last entry of `quests` is unnecessary and could disagree with the
canonical current quest. Reordering other sections is outside scope.

### Overlay the shared gauge on a reserved scrollable row

Remove task percentage from the compact-only `progress_text` summary. Reserve
one blank logical line immediately before it for the task gauge. Before
scrolling/rendering consumes the content, measure the wrapped prefix ending
before that reserved line at the exact Character inner width, using the same
`Paragraph` wrapping settings and rendered-line-count facility already used
for scroll bounds. This includes any prepended partial-eligibility lines and
wrapped activity text. Append the percentage summary after the reserved row.

Render the paragraph normally, including its reserved row and section
separators, then render `ProgressGauge` only if its content-row index lies in
the visible scroll window. Translate that index by the current clamped scroll
offset; draw within the inner rectangle at full inner width and height one.
Pass the existing `task_percent` argument and `Task <percent>%` label, clamping
fill as full mode does. Guard empty inner areas and off-screen gauge rows.

This keeps Ratatui responsible for text wrapping and preserves the existing
scroll machinery. A fixed row outside the paragraph would make the gauge
pinned rather than scrollable; manually approximating wrapping or making a
second text-based bar risks misplaced rows or divergent styling. A generic
mixed-content framework is unnecessary for one gauge.

## Risks / Trade-offs

- Wrapped content or eligibility text can shift the gauge row -> derive its
  position from the same paragraph wrapping configuration, not logical-line
  counts or string-length division; test wrapped prefixes and scrolled views.
- Extra separator rows consume vertical space -> retain existing combined
  scrolling and count all new rows when clamping offsets.
- Render order could overwrite text or borders -> reserve the row, overlay
  only inside the visible inner rectangle, and assert buffer contents/styles.
- An existing scrolling fixture may depend on long quest history to overflow ->
  make overflow come from retained content such as equipment, inventory, or
  spells; do not weaken the scrolling assertion.

No data migration is required. Rollback is limited to compact rendering,
related tests, and documentation; persisted state is unchanged.
