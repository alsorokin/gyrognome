# Tasks

## 1. Displayed-position model

- [x] 1.1 Add `floor_ms` to `TaskAnchor` (defaulting to `position_ms`) and a pure displayed-position function `max(anchored, floor_ms + 0.5 * anchored_advance)` clamped to duration. Use it for the displayed position and percent. Verify with a unit test of the spec's "Slowing down while ahead of the worker" numbers (6,500 ms at 1 s, caught up at 2 s).
- [x] 1.2 In `replace_character`, drop the rested-equality condition and set `floor_ms` to the currently displayed position for the same owned task. Verify with unit tests: same-task re-anchor with a lower persisted position does not decrease the bar, both without rest and with a changed rested balance; a new task resets to the persisted position.
- [x] 1.3 Keep the read-error re-anchor consistent with the new anchor (position and floor both set to the displayed value). Verify that the existing read-error test still passes.

## 2. Scheduling

- [x] 2.1 Make `next_task_percent_boundary` and `settling_deadline` follow the displayed curve (earliest of the two lines reaching the target). Verify with a unit test that a leading floor yields a later percent boundary and settling deadline than the raw anchored prediction.

## 3. Wrap-up

- [x] 3.1 Run `cargo test` and confirm that it passes, including the existing prediction and rested tests.
- [x] 3.2 Add a CHANGELOG entry noting that the task bar no longer snaps back, especially while rested.
