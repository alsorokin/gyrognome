# Design

## Context

See `proposal.md` for motivation. `src/dashboard.rs` already maintains independent
pane offsets in `PaneScroll`, detects task-completion changes through
`RecentTaskUpdates::between`, and routes both combined refreshes and shorter
persisted-state reads through `DashboardState::replace_character`.

Adventure currently renders inventory and spells as styled comma-separated
lines, separated by an empty line, with `Wrap { trim: true }`. When Journal is
collapsed, a current-quest line follows. `render_pane` measures wrapped content
with Ratatui's `Paragraph::line_count` and clamps offsets. Existing dashboard
tests use `TestBackend` and injectable `Instant` values.

## Goals / Non-Goals

**Goals:** Reuse existing task-update identity and pane scrolling; accurately
target entries within the existing wrapped inline layout; make timeout checks
deterministic without sleeping in tests.

**Non-Goals:** Changing list formatting to one entry per line, automatically
following Journal or compact content, adding persistent preferences, or
tracking individual simulation events between persisted reads.

## Decisions

### Detect new opportunities at the shared persisted-state boundary

In `replace_character`, compute the new completion interval's update set before
replacing the old character, as the highlighting path already does. Reuse its
inventory/spell `RowKey` identities, including duplicate-name occurrences.
Choose the first matching entry by iterating current display order, rather than
iterating a `HashSet`.

Treat each observed interval as one opportunity, not the lifetime of its retained
highlight. Create a transient Adventure reveal target only if the pause has
expired and the last rendered layout has expanded full-layout Adventure.
Clear stale targets on newer character replacements and consume a target on
the next render; if Adventure cannot be rendered then, discard it. Do not store
suppressed updates for later replay.

Alternatives: detecting changes on every render would repeatedly follow retained
highlights; adding a simulation-event subscription is unnecessary because both
refresh paths already share the required state boundary.

### Keep one session-local manual-scroll deadline

Store an optional Adventure auto-scroll-resume `Instant` in dashboard session
state. Pass the existing event timestamp through keyboard row/page and mouse
scroll routing to the common full-layout pane-scroll path. Each manual scroll
attempt directed at expanded Adventure sets the deadline to 30 seconds after
that timestamp and cancels any unrendered automatic reveal. Count attempts at
the top or bottom as manual intent, even when the clamped offset is unchanged.

Eligibility is checked when the persisted update is observed: `now >= deadline`
permits a new reveal. No timeout callback, queued update, or expiration redraw
is required. Other panes, compact scrolling, focus changes, and collapse
toggles leave this deadline alone.

Alternatives: resuming on a timer with the last highlighted entry would violate
the user's no-replay decision; suppressing until the user returns to the top
would not provide the requested 30-second pause.

### Resolve the target against actual Adventure wrapping

Retain `inventory_line`, `spells_line`, styling, separators, and optional quest
content. Add small Adventure-specific metadata connecting entry identities to
their spans and rendered row ranges at the current inner width. Wrapped-row
mapping must follow the same word wrapping, whitespace trimming, and terminal
cell widths as the rendered paragraph; character-count division or scrolling to
the inventory/spell section heading is insufficient.

Resolve this metadata during Adventure rendering, using current dimensions
rather than the previous layout's width. For a target that fits, adjust only
enough to include its complete row range; leave a fully visible target alone.
For a taller target, align its first row to the viewport top. Clamp the resulting
offset using the existing content-range bounds, without changing keyboard focus.
Keep this logic local to Adventure rather than changing the general behavior of
every `render_pane` call.

Alternatives: always scrolling to the top or bottom misses entries in the middle;
one-entry-per-line formatting simplifies targeting but unnecessarily changes
the established presentation.

## Risks / Trade-offs

- Wrapped inline spans can cross word and row boundaries -> assert actual
  `TestBackend` output for targets in both sections, including long entries,
  spaces, and non-ASCII cell widths; do not validate only the numeric offset.
- Several entries can change between reads -> reveal the first in display order,
  preserving existing highlighting for the others. No claim is made about which
  simulation event happened last.
- Manual scrolling can override a not-yet-rendered update -> cancel that target
  in the manual-scroll path so reading intent wins.
- Collapsed or compact views deliberately lose update-following opportunities ->
  discard them rather than unexpectedly jumping on a later layout change.

## Migration Plan

No data or configuration migration is needed. Reverting the dashboard changes
restores manual-only scrolling without affecting saved characters.
