# Tasks

## 1. Rested accounting and storage

- [x] 1.1 Add precise rested-duration accounting and a fallible Linux monotonic/boottime clock adapter; verify injected-clock unit tests cover the 43,200-second cap, stopped/sleep accrual, awake-delay exclusion, spending, fractional desktop periods, bank exhaustion, backwards wall time, and clock errors.
- [x] 1.2 Extend `src/runtime.rs` registration and SQLite migration with separate rested metadata initialized to zero at registration/migration time; verify synthetic browser/desktop migration and removal tests preserve canonical state, continuation, credentials, provenance, private-file protections, and profile-edit independence.
- [x] 1.3 Commit rested accounting atomically with browser state and desktop checkpoints, settle startup/stop timing, and add bounded periodic checkpoints; verify injected storage failures, graceful completion-free stops, restart, accounted-sleep recovery, and abnormal-exit tests retain complete matching checkpoints without replaying committed rewards.

## 2. Runtime acceleration

- [x] 2.1 Apply piecewise rested speed to the browser worker and schedule tick-aligned completion/exhaustion/checkpoint boundaries without minimum-tick inflation; persist fractional virtual time and verify fake-clock tests measure exactly 2x earned time with sufficient rest, 1,200 ms serviced plus 50 ms buffered for one second with 250 ms rest, normal-speed exhaustion, and canonical/RNG equivalence for equal complete tick sequences.
- [x] 2.2 Pace the desktop worker with a precise 1x/2x virtual clock while leaving callback rules intact; verify synthetic deadline tests measure 109.375 ms normal and 54.6875 ms boosted periods, correct exhaustion inputs, the unchanged 100 ms cap, full-bar-then-complete ordering, and skipped missed deadlines without bursts.
- [x] 2.3 Wire suspend accounting and awake-time spending through both worker loops, including delivery/persistence delays; verify injected samples show sleep earns rest once without immediate progression, stopped balances survive interruptions, and awake delays consume available rest without earning it or becoming catch-up.

## 3. Inspection and dashboard

- [x] 3.1 Expose available rest in milliseconds and active 1x/2x status through managed human/JSON inspection and safe dashboard snapshots; verify synthetic CLI/inspection tests cover active, stopped, exhausted, profile-edited, and export/reimport cases without credentials or read-induced accounting writes.
- [x] 3.2 Render rested status in full Details and compact Character content and update `TaskAnchor` prediction/redraw timing; verify dashboard tests cover doubled profile rates, split prediction at exhaustion, inactive/unknown ownership, re-anchoring on timing-only changes, saturation, scrolling, and unchanged pane heights.
- [x] 3.3 Simplify dashboard Details by removing ID and compatibility-profile rows, omit the rested-timeline acceptance notice from dashboard views, and show read-only projected rested time in character selection; cover rendering and selector storage integration.

## 4. Integration and documentation

- [x] 4.1 Preserve existing eligible online delivery and all current gates under the explicit rested-pacing exception; verify existing synthetic/fake-transport regressions retain event snapshots, commit-before-send ordering, no retries, local-only provenance, and unchanged development conformance pacing without adding or relabeling live evidence.
- [x] 4.2 Update README timing/runtime/dashboard guidance and the Unreleased changelog with automatic enablement, zero-bank migration, crash/clock limitations, and unverified online acceptance; verify the documentation matches delivered behavior and run focused Rust runtime/dashboard/inspection/report-boundary tests plus formatting checks. Do not run live leaderboard verification; the user will test their own characters separately.
