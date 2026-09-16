## Context

The terminal dashboard currently renders a fixed full-layout pane arrangement
and routes keyboard events through a command enum into dashboard state. Quest
history and the current quest are combined in the Adventure pane. See
`proposal.md` for motivation and
`specs/terminal-dashboard/spec.md` for required behavior.

## Goals / Non-Goals

**Goals:**

- Separate concise adventure context from historical quest information.
- Allow users to toggle individual full-layout panes without interrupting
  periodic refresh or lifecycle controls.
- Keep pane hotkeys discoverable where they apply.
- Preserve compact rendering and existing lifecycle command semantics.

**Non-Goals:**

- Add navigation, scrolling, filtering, persistence, or keyboard
  customization for quest history.
- Collapse the dashboard header, footer, or compact character view.
- Alter stored quest data, simulation behavior, or terminal size thresholds.

## Decisions

### Represent collapsible panes as dashboard UI state

Store the collapsed/expanded state in `DashboardState`, initialized with every
pane expanded. Map F1 through F7 to a fixed pane identity in input handling,
including F7 for Journal, then toggle only that identity when applying the
command.

This keeps presentation state local to a dashboard session and retains it
across periodic data refreshes. Encoding collapsed state into the snapshot
would incorrectly couple transient UI preference to persisted character and
service data; recomputing it only while rendering would not let key presses
change it.

### Build full-layout constraints from visible panes

Have the full renderer derive row and column constraints from the current
collapse state. Expanded panes receive their current fixed or flexible
allocation; collapsed panes render a header-only block with a minimal height
or are omitted from the relevant split so their former content space is
available to visible panes. When Adventure and Journal are both expanded,
allocate at most one quarter of the usable right-column height to Journal and
give Adventure the balance. Keep expanded Details at a fixed five-row
allocation, which leaves its two border rows and three inner-content rows.
Keep expanded Progress at a fixed seven-row allocation, leaving exactly five
inner rows for its five progress bars.

This satisfies the requirement that collapsing reduces allocation while
keeping pane state discoverable. A fixed layout that merely clears contents
would waste terminal space. Giving Journal an equal share makes its compact
history overtake the active adventure context, while a generic scrolling or
focusable layout would add unrequested interaction complexity.

### Centralize pane header construction

Use a shared pane-block/header helper that accepts a title and F-key label and
places the label at the right edge. Apply it to the seven collapsible full
dashboard panes, including Status, while leaving non-collapsible headers and
the compact view unchanged.

Centralizing this formatting prevents per-pane drift and provides consistent
shortcut discoverability. Adding each hint as body text would not meet the
header-placement requirement and would consume content space.

### Render quest data by audience

Retain existing inventory, spells, and plot output in Adventure but replace
its quest history with one plain `Current quest:` line. Render a new Journal
body with the bold current-quest value, without a label, first, followed by
the completed-quest collection in reverse order.

No model or persistence changes are needed because both values already arrive
in the dashboard snapshot. Duplicating completed quests in Adventure or
reordering stored quest data would preserve or create problems outside this
presentation-only change. Leading with the current quest keeps it visible in
the constrained Journal pane even when its completed history exceeds capacity.

## Risks / Trade-offs

- [Seven independently toggled panes can create sparse or unusually shaped
  layouts] → Derive constraints for every collapse-state combination and cover
  representative combinations in rendering tests.
- [F-key input may conflict with lifecycle confirmation flow] → Preserve
  existing confirmation behavior and specify the toggle command as a
  non-lifecycle state update.
- [Narrow terminal rendering could become inconsistent with full-layout
  controls] → Keep the established compact-width branch intact and test its
  content separately.

## Migration Plan

1. Release the dashboard rendering and input-state update as a backward-
   compatible CLI behavior change.
2. Users receive all panes expanded on each dashboard launch and can toggle
   them with the visible F-key hints.
3. Roll back by restoring the prior dashboard renderer and command mapping;
   no persisted data or migration is involved.
