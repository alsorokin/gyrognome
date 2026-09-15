## 1. Monotonic timing policy

- [ ] 1.1 Extract clock-independent worker elapsed-time selection that accepts prior and current monotonic instants plus the configured interval; verify unit tests cover on-time, shorter-than-interval, delayed, and zero elapsed durations.
- [ ] 1.2 Cap each scheduled worker update at one configured interval and reset the timing baseline after every callback attempt; verify delayed callbacks do not accumulate discarded time into a later update.
- [ ] 1.3 Preserve existing explicit-duration overflow handling and atomic persistence semantics; verify an unrepresentable duration and a simulation failure leave the last persisted canonical state unchanged.

## 2. Deterministic conformance proof

- [ ] 2.1 Add simulation tests proving a total elapsed duration and equivalent nonzero partitions produce identical canonical state and Alea continuation before and across a task-completion boundary.
- [ ] 2.2 Add sanitized paired conformance checkpoints derived from disposable browser observations for elapsed-duration partitioning and task completion; verify the checkpoint runner replays both sequences exactly.
- [ ] 2.3 Extend fixture-safety coverage for the new timing fixtures; verify they contain no player saves, passkeys, browser profiles, or signed leaderboard requests.

## 3. Runtime integration and delivery

- [ ] 3.1 Integrate the capped elapsed-time selector into the worker loop without changing the configured default cadence or stopped-runtime behavior; verify integration tests cover normal progression, restart without downtime catch-up, and capped scheduler delay.
- [ ] 3.2 Update runtime documentation to describe monotonic timing, capped delayed callbacks, and the no-catch-up policy; verify documented commands against an isolated synthetic runtime data directory.
- [ ] 3.3 Run formatting, warning-denied linting, deterministic simulation checkpoint tests, local runtime tests, fixture-safety tests, and the complete test suite; verify all commands complete without warnings or failures.
