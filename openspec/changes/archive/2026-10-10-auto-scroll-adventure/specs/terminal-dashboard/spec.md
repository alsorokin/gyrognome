# Spec Delta

## ADDED Requirements

### Requirement: Adventure update auto-scroll

When a persisted refresh observes one or more completed tasks with newly added
or changed inventory items or spells, the dashboard SHALL automatically reveal
the first such entry in Adventure display order, provided Adventure is expanded
in the full layout and its manual-scroll pause has expired. Inventory entries
SHALL precede spells for target selection, retaining their existing display
order. Removed entries SHALL NOT be reveal targets.

The dashboard SHALL use the entry's rendered wrapped rows to bring it into
view with the smallest necessary vertical movement. It SHALL keep an already
fully visible target in place. If an entry exceeds the viewport height, it
SHALL show the beginning of that entry. The scroll offset SHALL remain within
the available content range.

Every manual keyboard or mouse-wheel scroll directed at expanded full-layout
Adventure SHALL pause Adventure auto-scroll for 30 seconds of monotonic elapsed
time, including scroll attempts at a content boundary. Each further manual
Adventure scroll SHALL restart that interval. Scrolling another pane, changing
keyboard focus, or toggling a pane SHALL NOT restart the interval.

Updates observed during the pause SHALL NOT be queued or replayed. Expiration
alone SHALL NOT move the viewport; only a qualifying update observed at or
after expiration SHALL trigger another automatic reveal. An initial snapshot,
an unchanged refresh, or a redraw retaining an existing update highlight SHALL
NOT trigger auto-scroll.

Automatic reveals SHALL NOT change keyboard focus, pane collapse state, other
pane offsets, or the compact Character offset. Updates observed while Adventure
is collapsed or the dashboard is compact SHALL NOT be replayed on expansion or
return to the full layout. Journal behavior SHALL remain unchanged. This
behavior SHALL remain display-only and SHALL NOT advance simulation, persist
character state, or issue network requests.

#### Scenario: Revealing an off-screen new entry

- **WHEN** a persisted task completion adds an inventory item outside the
  expanded Adventure viewport and no manual-scroll pause is active
- **THEN** Adventure scrolls the item's rendered rows into view with the smallest
  necessary offset change
- **AND** keyboard focus and other pane offsets remain unchanged

#### Scenario: Revealing a changed spell

- **WHEN** a persisted task completion changes an off-screen spell rank without
  changing any inventory entry and Adventure auto-scroll is enabled
- **THEN** Adventure reveals that spell without changing the inline list layout

#### Scenario: Selecting among several updates

- **WHEN** one persisted task-completion refresh adds or changes several
  inventory items and spells
- **THEN** Adventure selects the first changed inventory entry in display order,
  or the first changed spell if no inventory entry changed

#### Scenario: Keeping a visible target in place

- **WHEN** a qualifying update's selected entry is fully visible
- **THEN** Adventure retains its current offset

#### Scenario: Revealing an entry taller than the viewport

- **WHEN** the selected updated entry wraps across more rows than Adventure can
  display at once
- **THEN** Adventure shows the beginning of that entry within its valid scroll
  range

#### Scenario: Respecting and extending manual scrolling

- **WHEN** the user manually scrolls expanded Adventure at time T and scrolls
  it again at T plus 20 seconds
- **THEN** qualifying updates observed before T plus 50 seconds do not cause
  automatic scrolling
- **AND** scroll attempts at a content boundary also restart the pause

#### Scenario: Resuming only for updates after expiration

- **WHEN** an update is observed during the manual-scroll pause and the pause
  subsequently expires without another qualifying update
- **THEN** Adventure does not move or replay the suppressed update
- **AND** a qualifying update observed at or after the expiration resumes
  automatic revealing

#### Scenario: Scrolling another pane

- **WHEN** the user scrolls Journal or another pane, or only changes scroll focus
- **THEN** Adventure's manual-scroll pause is not started or extended

#### Scenario: Refreshing without a new update

- **WHEN** the dashboard opens, refreshes without another completed-task update,
  or redraws an existing highlighted value
- **THEN** Adventure does not automatically change its offset

#### Scenario: Observing updates while Adventure is hidden

- **WHEN** a qualifying update is observed while Adventure is collapsed or the
  dashboard is compact
- **THEN** no automatic reveal occurs
- **AND** expanding Adventure or returning to full layout does not replay that
  update or change the compact scroll offset
