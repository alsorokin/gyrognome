## 1. Canonical simulation foundations

- [x] 1.1 Consolidate the parsed-character and RNG Alea state types into one browser-compatible representation; verify existing RNG continuation and save-import tests retain their expected values.
- [x] 1.2 Define a pure simulation API that accepts immutable canonical state, an explicit ruleset, and supplied elapsed milliseconds and returns a new state; verify it has no filesystem, clock, database, or HTTP dependencies.
- [x] 1.3 Implement browser-compatible progress-bar operations and capped 100 ms advancement slices; verify incomplete-task advancement changes only the task bar and leaves task count, elapsed time, rewards, and Alea state unchanged.

## 2. Ruleset and conformance evidence

- [x] 2.1 Capture the browser rule tables required for task selection, combat, rewards, equipment, spells, quests, plots, and level-ups as ordered, versioned bundled data with source provenance; verify the selected ruleset is immutable for a simulation run.
- [x] 2.2 Add a safe synthetic checkpoint schema recording initial canonical state, ruleset revision, elapsed-time inputs, expected canonical state, and expected Alea continuation; verify fixture-safety checks reject prohibited player or leaderboard data.
- [x] 2.3 Record disposable-browser-derived checkpoints for incomplete advancement and a completed task; verify replay tests compare full canonical state and Alea continuation exactly. (Incomplete-advancement checkpoint has a full exact-replay test now. The completed-task checkpoint is captured and committed, but its exact-replay assertion is deferred to 3.1 because exact replay depends on the completion-dispatch logic that task ports; a schema/data-level test covers it in the meantime. Confirmed with user.)

## 3. Browser task-completion behavior

- [x] 3.1 Port browser task queue selection and completion dispatch in browser order; verify a completed-task checkpoint updates task count, elapsed time, activity, and random state exactly. (Ports `Timer1Timer`'s completion bookkeeping — task count, elapsed time, the "gain"-gated experience/quest/plot advancement — and `Dequeue`'s plain task/plot queue-selection path, exactly reproducing the deferred checkpoint-completed-task.json fixture from 2.3 including its plot-bar advancement. Branches that require combat resolution or reward mechanics not yet ported — kill-task loot, monster-task selection, buying/market/selling, level-up, quest completion, interplot cinematics, and act completion — report `SimulationError::Unsupported` rather than fabricating results; each is covered by tasks 3.2/3.3.)
- [ ] 3.2 Port combat resolution and its experience, quest, and plot progress effects; verify combat checkpoints cover non-leveling and progress-bar-boundary outcomes. (Also covers monster-task selection, which `Dequeue` needs to pick the next "kill" task.)
- [ ] 3.3 Port level-up, attribute, spell, equipment, inventory, quest, and plot/act reward paths using the selected ruleset; verify each behavior family has a browser-derived checkpoint with an exact state/RNG match.

## 4. Integration and documentation

- [ ] 4.1 Preserve the read-only `inspect` workflow while exposing the pure simulation API only to library callers; verify simulation does not modify the imported raw save document or inspection output.
- [ ] 4.2 Document the selected ruleset provenance, deterministic advancement contract, checkpoint process, and continued absence of scheduling, persistence, UI, and transport; verify example conformance commands run with synthetic fixtures.
- [ ] 4.3 Run formatting, warning-denied linting, fixture-safety tests, and the complete test suite; verify the project reports no warnings or failures.
