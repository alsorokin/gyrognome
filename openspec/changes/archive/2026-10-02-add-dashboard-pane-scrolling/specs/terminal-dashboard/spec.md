# Spec Delta

## ADDED Requirements

### Requirement: Scrolling overflowing dashboard content

The dashboard SHALL allow users to read vertically overflowing content in
expanded full-layout panes and in the compact layout's combined Character
pane. In the full layout, Tab and Shift+Tab SHALL move keyboard scroll focus
forward and backward among expanded panes, wrapping at either end. The
focused pane SHALL be visibly distinguishable. Up and Down SHALL scroll the
focused pane by one content row; PageUp and PageDown SHALL scroll it by one
visible content page. Scrolling SHALL NOT alter character state or pane
collapse state.

When the user scrolls the mouse wheel over an expanded full-layout pane, the
dashboard SHALL scroll that pane. In the compact layout, keyboard and mouse
wheel scrolling SHALL apply to the combined Character pane.

Scroll offsets SHALL be independent for each pane and SHALL be kept within the
available content range as the terminal is resized or content changes.

#### Scenario: Navigating keyboard scroll focus

- **WHEN** the user presses Tab or Shift+Tab in the full dashboard
- **THEN** keyboard scroll focus moves to the next or previous expanded pane,
  wrapping at the first and last panes
- **AND** the focused pane is visibly distinguished

#### Scenario: Scrolling a focused full-layout pane

- **WHEN** an expanded full-layout pane has more rendered content rows than
  its visible area and the user presses Up, Down, PageUp, or PageDown
- **THEN** only the focused pane scrolls in the requested direction
- **AND** its content remains within the available scroll range

#### Scenario: Scrolling with the mouse wheel

- **WHEN** the user moves the mouse wheel over an expanded full-layout pane
- **THEN** only the pane under the pointer scrolls
- **AND** its scroll offset remains within the available content range

#### Scenario: Scrolling compact Character content

- **WHEN** the compact dashboard's Character content exceeds its visible area
- **THEN** keyboard scrolling and mouse-wheel scrolling allow the user to read
  all content in that pane

#### Scenario: Keeping scroll position valid

- **WHEN** terminal resizing or refreshed content reduces the available
  content range of a pane
- **THEN** that pane's scroll offset is clamped to a valid position
- **AND** offsets for other panes remain unchanged

#### Scenario: Keeping dashboard actions and state safe

- **WHEN** the user scrolls or changes keyboard scroll focus
- **THEN** no character state is changed and existing pane collapse shortcuts
  continue to work
