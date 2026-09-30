# Design

## Context

The full dashboard currently divides the screen into header, content, Status,
and Keys rows. Status participates in the seven-entry `Pane`/`PaneVisibility`
model and consumes F5, so Adventure and Journal use F6 and F7. The Adventure
content independently formats a plot line from the numeric act and canonical
plot caption, even though the caption already carries presentation text such as
`Act VIII`.

The compact dashboard has separate content and sizing behavior and must retain
its current plot and Status presentation, while its Keys height now follows the
same fit-versus-wrap rule as the full layout. See `proposal.md` for motivation
and `specs/terminal-dashboard/spec.md` for the observable contract.

## Goals / Non-Goals

**Goals:**

- Represent only user-collapsible panes in the pane visibility and hotkey model.
- Use the persisted canonical plot caption as the Journal title suffix rather
  than maintaining a second Roman-numeral conversion path in dashboard code.
- Give Keys and Status predictable side-by-side allocations that still allow
  their text to wrap at the full-layout width threshold.
- Keep version declarations synchronized across Rust and Node metadata.

**Non-Goals:**

- Changing compact-dashboard content, the full-layout width threshold, or
  terminal minimum dimensions.
- Changing lifecycle actions, status messages, confirmation behavior, or
  character simulation and persistence.
- Renaming the Adventure or Journal concepts beyond the dynamic Journal title.

## Decisions

### Remove Status from the collapsible pane model

`Pane` and `PaneVisibility` will contain six entries in visual shortcut order:
Activity, Progress, Equipment, Details, Adventure, and Journal. Command mapping
and header labels will assign these panes F1 through F6. Status will render with
a normal titled block rather than `pane_block`, which prevents both accidental
collapse state and a misleading hotkey label.

Alternative considered: retain Status in `Pane` but ignore its visibility flag.
That would leave an invalid state and require special cases in command mapping,
tests, and header rendering.

### Build a dedicated full-layout bottom row

The top-level full layout will allocate header, main content, and one bottom
row. That bottom row will be split horizontally into Keys and Status areas in
left-to-right order using the same one-third/two-thirds ratio as the main
dashboard columns. In the normal state and ordinary confirmation state, the
bottom row will be three terminal rows tall when the Keys content fits its
inner width and four rows tall only when that content wraps. Both panes share
the selected height so their borders stay aligned. The wider Status pane
retains the full `Runtime ownership` label.
The Keys line will use styled spans: each shortcut key is bold and each group
is separated by ` | `.
A confirmation with safety warning text will use a taller eight-row allocation
so the narrower Keys pane still displays the warning. Compact layout will
continue to use vertically stacked Status and Keys rows, but will render the
same styled shortcut line and select a three- or four-row Keys allocation based
on whether that line fits the pane's inner width.

Alternative considered: size Keys to its content. Reusing the main-column ratio
creates consistent vertical alignment with the panes above at the cost of
wrapping the shortcut line on narrower terminals; responsive row height avoids
paying that vertical cost when the line fits.

### Derive the Journal title from the canonical plot caption

The full-layout Journal title will be formatted as `Journal - {bestplot}` and
passed through the existing pane block renderer. `adventure_lines` will retain
inventory, the separating blank line, spells, and the conditional current
quest, but will no longer append blank space and a plot row. This reuses values
such as `Prologue` and `Act VIII` exactly as stored and avoids duplicating the
simulation's Roman numeral formatting.

Alternative considered: convert `plot.act` to Roman numerals in the dashboard.
That duplicates canonical formatting already represented by `bestplot` and
risks disagreement with imported desktop captions.

### Keep release metadata changes mechanical

The patch release will update `Cargo.toml`, the root `gyrognome` package entry
in `Cargo.lock`, `package.json`, and both root package version fields in
`package-lock.json` from `1.1.0` to `1.1.1`. Dependency versions remain
unchanged.

## Risks / Trade-offs

- [Long plot captions reduce space available for the right-aligned F6 label] →
  Rely on Ratatui's title rendering behavior and add rendering coverage for the
  representative `Journal - Act VIII` title and hotkey.
- [Side-by-side panes wrap at the minimum full-layout width] → Allocate a
  multi-line bottom row and test both the 90-column threshold and the standard
  120-column layout.
- [Removing Status shifts established shortcuts] → Update command tests,
  rendered header assertions, visible README documentation, and ensure F7 is no
  longer handled.
- [Moving Status changes space available to the main content] → Assert that the
  main content receives the reclaimed rows and preserve the existing internal
  pane constraints.

## Migration Plan

No data migration is required. Ship the layout and shortcut changes together
with the patch-version bump. Rollback consists of reverting the release; saved
characters and runtime service state are unaffected.
