# Design

## Context

See proposal.md for motivation and
`specs/terminal-dashboard/spec.md` for the observable contract. The dashboard
currently renders fixed-size panes as Ratatui Paragraphs, maps F1-F6 to
collapse toggles, and uses Crossterm events in one terminal session loop.
Compact mode renders all character information in one Character pane.

## Goals / Non-Goals

**Goals:**

- Scroll wrapped content by rendered terminal rows without changing its
  formatting or the existing pane geometry.
- Keep keyboard focus and scroll offsets stable across redraws, pane toggles,
  refreshes, and terminal resizes.
- Route wheel input to the pane under the pointer and restore terminal mouse
  mode on every dashboard exit.

**Non-Goals:**

- Add horizontal scrolling, touchpad-specific gestures, or mouse-click focus.
- Change compact/full layout thresholds, pane sizing, or character content.
- Persist scroll position outside the active dashboard session.

## Decisions

### Keep scroll state in the dashboard session

Store a vertical offset for each full-layout pane and one for compact
Character, plus the currently focused full-layout pane. Offsets remain
independent when panes are collapsed and expanded. When focus is on a pane
that becomes collapsed, move it to the next expanded pane. Start focus at the
first expanded pane.

Alternative considered: keep only a single shared offset. Per-pane state
allows users to return to the same place when navigating between quest
history, details, and other content.

### Scroll rendered rows inside existing panes

Apply offsets to the pane's wrapped Paragraph content, using the pane's inner
height as the page size. Clamp each offset to the maximum rendered-content
overflow after layout or refreshed-content changes. Preserve the current
wrapping and layout constraints rather than expanding a pane to fit its
content.

Up and Down move one row; PageUp and PageDown move by one inner viewport.
In compact mode those same keys always target Character. Tab and Shift+Tab
cycle focus only in the full layout. Existing F1-F6 collapse commands remain
available.

Alternative considered: focus and scroll only panes that currently overflow.
Including every expanded pane keeps focus order stable as content changes;
scrolling a pane without overflow simply remains at its current boundary.

### Route mouse-wheel events using rendered pane bounds

Enable Crossterm mouse capture for the dashboard session. Use the rectangles
already produced by the layout to locate the pane under each wheel event; only
expanded full-layout panes are wheel targets. In compact mode, the Character
rectangle is the only target. A wheel event scrolls its target without
changing keyboard focus. Disable mouse capture during terminal-session
cleanup, alongside restoring raw mode and the alternate screen.

Alternative considered: move keyboard focus to the pane under the pointer.
Keeping pointer and keyboard focus independent avoids surprising focus jumps
while the user continues keyboard navigation.

### Mark keyboard focus without consuming the hotkey label

Use a subtle border or title style for the focused pane while retaining the
right-aligned F1-F6 label. Keep the compact Character pane's normal title;
it is the only scroll target there and does not need a focus cycle.

Alternative considered: add a focus indicator to the title text. Styling the
existing border avoids reducing title space or changing the displayed title.

## Risks / Trade-offs

- [Some terminal emulators may not report wheel events] -> Keyboard scrolling
  remains complete and available without mouse input.
- [Mouse capture may be left enabled if cleanup is incomplete] -> Restore it
  in the existing terminal-session drop path.
- [Wrapped content makes offsets differ from source-line counts] -> Measure
  the rendered Paragraph row count at the current pane width and clamp against
  its inner viewport height.

## Migration Plan

No migration is needed. Scroll state exists only for the lifetime of a
dashboard session, and no dependency or persisted data changes are expected.
