# Tasks

## 1. Keyboard Scroll State and Rendering

- [x] 1.1 Add full-layout pane focus and independent scroll offsets, preserve offsets across pane toggles, and map Tab/Shift+Tab plus Up/Down/PageUp/PageDown; verify focus order, wraparound, and boundary behavior with unit tests
- [x] 1.2 Apply clamped rendered-row offsets and a visible focus style to full-layout panes and compact Character; verify wrapped content can be reached and scroll state stays valid after resizing or refreshed content

## 2. Mouse Scrolling and Documentation

- [x] 2.1 Enable mouse capture for the dashboard session, route wheel events to the pane under the pointer, and restore mouse mode during cleanup; verify full and compact target selection and wheel direction with tests
- [x] 2.2 Document keyboard and mouse scrolling controls in README.md and run `cargo test dashboard::tests`
