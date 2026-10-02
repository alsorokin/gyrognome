# Proposal

## Why

Long dashboard content is clipped by the fixed pane allocations, so users
cannot read all quest history, wrapped lists, or variable details. Vertical
scrolling will make that content accessible in both full and compact layouts.

## What Changes

- Add keyboard focus and vertical scrolling for expanded full-layout panes.
- Add mouse-wheel scrolling for the pane under the pointer.
- Make the compact layout's combined Character pane scrollable with both input
  methods.
- Preserve existing pane collapse shortcuts and dashboard content.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `terminal-dashboard`: Specify keyboard and mouse-wheel scrolling for
  overflowing full-layout panes and the compact Character pane.

## Impact

- `src/dashboard.rs`: dashboard input handling, pane focus, scroll offsets,
  and rendering.
- Dashboard rendering and input tests in `src/dashboard.rs`.
- `README.md`: document scrolling controls.
- `openspec/specs/terminal-dashboard/spec.md`: update the dashboard behavior
  contract.

No dependency or persisted-state changes are expected.
