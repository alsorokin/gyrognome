# Design

## Context

The dashboard currently maps persisted state into `DashboardSnapshot` and
replaces the current snapshot after every successful provider refresh. Its
rendering is stateless: compact output is formatted text, while the full
Adventure and Equipment panes use Ratatui lines and paragraphs. It currently
does not expose stats in either layout.

The official client clears list selections at the start of every completed
task, then selects rows through the same mutations that update stats, spells,
equipment, and inventory. Dashboard refreshes are read-only and operate from
persisted canonical state, so the equivalent indicator must be derived from
successive snapshots rather than written to the character or simulation.

## Goals / Non-Goals

**Goals:**

- Identify and style currently rendered stat, inventory, spell, and equipment
  values changed after an observed task completion.
- Keep the selected values stable across ordinary refreshes and replace them
  at the next observed completion, matching the official client's
  clear-then-select behavior.
- Make stats and selection state visible in both supported layouts without
  adding a dashboard side effect.

**Non-Goals:**

- Persist selection metadata, alter canonical saves, or modify simulation
  advancement.
- Highlight task-progress, activity, quest, plot, identity, or service-status
  changes.
- Reconstruct intermediate per-task mutations when an external runtime writes
  multiple completions between two dashboard refreshes; in that case the
  visible comparison is the two observed persisted states.

## Decisions

### Keep a display-only previous snapshot and update indicator state on refresh

`DashboardState` will retain the prior displayed character data and a
display-only collection of changed keys. A successful refresh whose completed
task count increased compares the prior and new state and replaces the
collection; a successful refresh without a task-count increase preserves it.
Provider errors leave both the currently rendered snapshot and indicator
unchanged.

This preserves the observer boundary and corresponds to the official
client's clearing selections only on task completion. Re-diffing every
refresh was rejected because it would erase the indicator immediately after a
completed task. Persisting markers in the character was rejected because
browser-compatible canonical state has no dashboard-selection field.

### Compare values by their rendered logical identities

Stats and equipment will be compared by their fixed field/slot identities.
Inventory and spells will be compared by their names, including additions and
changed quantity or rank. The renderer will style only values present in the
new snapshot; removed rows naturally disappear, as they do in the official
client. This keeps selection independent of display order and avoids
highlighting unrelated list positions after insertion or removal.

Comparing whole collections or positions was rejected because inventory
reordering during selling and new list entries could make unchanged values
appear selected.

### Place stats in the responsive header

The dashboard will use a single named highlight style consistent with its
existing terminal selection treatment. The header will render the identity on
the left and stats on the right when both fit its inner width. Otherwise, it
will expand by one content row and place stats below identity. This preserves
the documented full-layout pane geometry and shortcuts while keeping stats
visible in compact and full layouts.

Adding a pane was rejected because it would introduce a new layout control and
change the existing constrained layout beyond the requested value indicator.
Displaying stats in Adventure was rejected because stats describe the
character, not a specific adventure event.

## Risks / Trade-offs

- [Multiple task completions occur between refreshes] → Compare the last
  displayed and current persisted states, document the observed-interval
  boundary, and replace rather than accumulate highlight state at that
  refresh.
- [Long values reduce compact readability] → Preserve existing wrapping and
  emit each formatted list section as styled text, without changing the
  compact-layout threshold.
- [A rendered collection contains duplicate names] → Use stable occurrence
  handling in comparison so equivalent duplicate entries are not spuriously
  highlighted.

## Migration Plan

No data migration is required. The change is presentation-only and can be
rolled back by removing the dashboard comparison and styling state; persisted
characters and runtime behavior remain compatible.
