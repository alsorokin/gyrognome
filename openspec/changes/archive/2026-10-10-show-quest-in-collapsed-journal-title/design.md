# Design

## Context

See `proposal.md` for motivation. Full-layout rendering in `src/dashboard.rs`
currently passes Journal's collapse state to `adventure_lines`, which appends a
blank line and labeled current quest. Journal's title always includes
`state.plot.bestplot`; `render_pane` already renders that title when collapsed,
and `pane_block` adds a right-aligned F6 shortcut.

`DashboardCharacter.current_quest` already supplies the canonical browser or
desktop quest caption. Expanded `journal_lines` and compact Character content
consume the same value independently. No new state or quest derivation is
needed.

## Goals / Non-Goals

**Goals:** Keep this a display-only change using existing collapse state,
canonical captions, and pane rendering. Resolve title formatting and narrow
header behavior without adding a separate layout mechanism.

**Non-Goals:** Change compact rendering, Journal history ordering, pane heights,
scroll focus, Adventure auto-scroll, or simulation and refresh scheduling.

## Decisions

- Build the Journal title from the displayed snapshot on every full-layout
  render. Use `Journal - <plot>` when expanded or when the quest is empty, and
  `Journal - <plot> - <quest>` otherwise. Retaining the plot prefix instead of
  replacing it preserves existing context; adding a `Current quest:` label
  would consume scarce header width without adding meaning.
- Remove `show_current_quest` from `adventure_lines` and update its callers,
  including auto-scroll tests. Adventure remains its existing three logical
  lines: inventory, blank separator, spells. Keeping a permanently false flag
  would leave obsolete behavior and misleading tests.
- Reuse the existing bordered pane title and right-aligned hotkey. Let long
  titles clip rather than wrap or introduce horizontal scrolling. Check actual
  rendered output for F6 visibility; if existing title overlap obscures it,
  constrain the Journal title to the space before F6 using terminal display
  width, without changing other panes.
- Leave expanded `journal_lines` and compact Character content untouched.
  The delta replaces the existing managed-dashboard requirement in full so
  archive will not discard unrelated selection and error behavior.

## Risks / Trade-offs

- [Long plot and quest captions may leave little visible quest text] ->
  Accept existing header clipping and preserve F6; users can expand Journal
  to read the full quest.
- [Removing the quest tail changes Adventure's maximum scroll offset] ->
  Keep the existing offset clamp and inventory/spell row mapping; exercise
  nearby Adventure auto-scroll tests.
- [A title test alone could miss a duplicate Adventure quest row] ->
  Assert both the rendered title and Adventure content, including collapse,
  expansion, refresh, and the compact-layout regression.

## Migration Plan

No data migration or configuration change is required. Ship the rendering
change normally; rollback restores the prior rendering without touching
character state.
