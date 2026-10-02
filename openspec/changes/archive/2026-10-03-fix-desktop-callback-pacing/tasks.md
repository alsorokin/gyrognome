# Tasks

## 1. Desktop worker pacing and persistence

- [x] 1.1 Add `DESKTOP_CALLBACK_PERIOD` (109,375 µs) beside `MAX_CALLBACK_ELAPSED_MS` and replace desktop sleep-after-work with a fixed-rate deadline scheduler that skips missed deadlines; verify with a unit test of the deadline-advance helper covering on-time and overrun cases.
- [x] 1.2 Take the desktop timing baseline right after the simulation step, before commit and report I/O; verify with a test that a slow commit does not reduce the next callback's credited milliseconds below 100 when it fires on schedule.
- [x] 1.3 Cache the desktop checkpoint and pending counters in `Worker`, generalize `replace_desktop_checkpoint` to accept accumulated credited milliseconds and completed tasks, and commit on completion, before reports, on newly required local-only provenance, and on graceful stop in `run_until`, while keeping `advance_elapsed` committing every call; verify with tests that partial progress is not committed per callback, that completion and report callbacks commit before delivery, and that a stop flushes the latest state, including a full pending bar.
- [x] 1.4 Re-read authentication and evidence availability at each commit; verify existing desktop local-only provenance and reporting tests still pass (`cargo test runtime::`).

## 2. Dashboard prediction rate

- [x] 2.1 Add a per-profile prediction rate to `TaskAnchor` (1.0 browser, 100/109.375 desktop) and apply it in predicted position, whole-percent redraw boundaries, and the settling deadline; verify with dashboard tests that a desktop task with 10,000 ms remaining saturates after about 10,938 ms and browser behavior is unchanged (`cargo test dashboard::`).

## 3. Docs and check

- [x] 3.1 Update the desktop timing section of `docs/classic-desktop-compatibility.md` with the 109.375 ms cadence and the commit points; verify by reading the rendered section.
- [x] 3.2 Reinstall the binary, restart the Pemptus worker, and sample `measured_since_import.elapsed_milliseconds` over 60 s of wall time; verify it advances by about 54–55 s, and the dashboard bar no longer visibly sticks at 100%.
