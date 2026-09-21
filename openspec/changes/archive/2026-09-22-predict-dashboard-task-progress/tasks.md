# Tasks

## 1. Worker boundary alignment

- [x] 1.1 Add a pure function in the runtime returning the simulated duration until the current task completes, rounded up to the simulation's 100 ms tick grid and floored at one tick; verify unit tests cover an exact tick multiple, a remainder that rounds up, an already-complete task, and a zero-duration task all yielding whole tick multiples of at least one tick.
- [x] 1.2 Change the worker's sleep to the lesser of that aligned remaining duration and its configured interval, advancing by the same duration through the existing `select_worker_elapsed` cap; verify a test drives several wakes across a task boundary and asserts the boundary wake lands exactly on the task's aligned completion.
- [x] 1.3 Confirm the aligned worker persists a completed task in the same wake that completes it, so the next persisted snapshot shows the following task near its starting position; verify a runtime test reads the store after the boundary wake and asserts the task identity advanced.
- [x] 1.4 Confirm advancement remains identical to the previous fixed-interval behavior; verify `cargo test` passes the timing checkpoint fixtures and a test asserts an aligned wake sequence and a fixed-interval sequence over the same total duration produce the same canonical state and Alea continuation.

## 2. Prediction arithmetic

- [x] 2.1 Add a display-only task anchor value in the dashboard capturing the observed task position, task duration, task identity (completed-task count and duration), and the `Instant` at which the state was observed; verify a unit test constructs an anchor from a snapshot without touching the store, a lock, or the simulation.
- [x] 2.2 Add a pure predicted-position function over an anchor and an elapsed `Duration` that clamps to the task duration; verify unit tests cover mid-task advancement, exact saturation, elapsed time beyond the duration, and a zero-duration task treated as immediately saturated.
- [x] 2.3 Add a pure predicted-percent function matching the browser-compatible floor used by `ProgressBar::reposition`; verify unit tests assert the predicted percent equals the persisted percent at zero elapsed time and reaches exactly 100 at and beyond saturation.
- [x] 2.4 Add a pure next-percent-boundary function returning the duration until the predicted bar would display the next whole percent, returning no deadline when saturated or when the task duration is zero; verify unit tests cover a long task, a one-second task, the final percent step, and the saturated case.
- [x] 2.5 Apply the configured minimum redraw spacing as a floor on the boundary deadline; verify a unit test shows a one-second task never requests a deadline shorter than the floor while a long task keeps its exact per-percent deadline.

## 3. Provider and refresh split

- [x] 3.1 Add a state-only read to `DashboardProvider` alongside the existing combined `refresh`, implement it on `LocalProvider` by reading persisted character state without invoking the service runner, and update the in-module test provider; verify `cargo test dashboard` passes with the test double recording state-only and combined reads separately.
- [x] 3.2 Update the dashboard state so a state-only read replaces the displayed character data, re-anchors the prediction, and preserves the last known service status and ownership; verify a test asserts a state-only read leaves service status untouched and performs no service-runner call.
- [x] 3.3 Preserve existing failure behavior for the state-only read by keeping the last successfully displayed character state and surfacing an actionable message; verify a test drives a failing state-only read and asserts the previous state still renders.

## 4. Run-loop scheduling

- [x] 4.1 Replace the fixed `event::poll` timeout with a computed deadline that is the minimum of the next percent boundary, the next scheduled combined refresh, and the next permitted settling-triggered read, capped at the configured refresh interval; verify a unit test over the deadline selection covers each of the three sources winning.
- [x] 4.2 Trigger a state-only read a bounded settling interval after the prediction saturates, gated on runtime ownership, with the interval sized to cover a store commit rather than a worker interval; verify a test asserts no read is issued at the saturation instant, that one is issued after the settling interval and before the regular interval elapses, and that no service-runner call occurs.
- [x] 4.3 Separate successive settling-triggered reads by at least the settling interval while the persisted task is unchanged; verify a test drives repeated saturated reads that return the same task and asserts the read count over a fixed elapsed span stays within that bound.
- [x] 4.4 Stop predicting and stop triggering settling reads when the character is not owned or ownership is unknown; verify a test asserts a non-owned snapshot renders the persisted task percent unchanged across elapsed time and issues no triggered reads.
- [x] 4.5 Hold the displayed task position non-decreasing while the task identity is unchanged, and adopt the persisted position directly when the task identity changes; verify unit tests cover a re-anchor to a lower position within one task and a reset at a task boundary.
- [x] 4.6 Keep the existing quit, interrupt, explicit-refresh, pane-toggle, lifecycle-confirmation, and brag behavior working against the computed deadline, with the existing `r` explicit refresh still performing a combined state and service read; verify `cargo test dashboard` passes including the existing observer-boundary and lifecycle-failure tests.

## 5. Rendering

- [x] 5.1 Thread a display-only predicted task percentage into rendering, defaulting to the persisted percentage when prediction is inactive, without adding the predicted value to `DashboardSnapshot`; verify a Ratatui buffer test renders a predicted percent in the full Progress pane while the snapshot retains its persisted value.
- [x] 5.2 Use the same predicted task percentage in the compact progress summary; verify a buffer test at compact width shows the predicted task percent alongside unchanged experience, encumbrance, plot, and quest percents.
- [x] 5.3 Confirm the recent-task-update indicator, lifecycle actions, and the manual-brag path continue to read only persisted state; verify existing tests for the update indicator and brag outcome still pass unchanged.

## 6. Validation and documentation

- [x] 6.1 Add a dashboard test that advances simulated elapsed time across a task boundary and asserts the bar advances predictively, holds at 100% through the settling interval until new state is read, then re-anchors to the next task's persisted values; verify `cargo test dashboard` passes.
- [x] 6.2 Update the README terminal dashboard section to describe predicted task-bar motion, its confinement to the task bar, that it applies only while a runtime owns the character, and the unchanged `--refresh-ms` default and bounds; verify the section matches the shipped defaults in `src/cli.rs`.
- [x] 6.3 Update the README worker description to note that the worker wakes at the earlier of its interval and the current task's completion, and that `--interval-ms` remains an upper bound on how long it sleeps; verify the text matches the shipped default in `src/cli.rs`.
- [x] 6.4 Run `cargo fmt --check` and the full `cargo test` suite and confirm no simulation, runtime, persistence, lifecycle, or reporting behavior regressed; verify both commands succeed.
