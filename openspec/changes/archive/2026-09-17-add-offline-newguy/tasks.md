## 1. Offline character generation

- [x] 1.1 Add a pure offline character generator that validates bundled-ruleset race and class selections, produces valid level-one canonical state from fresh local randomness, and verify generator unit tests cover traits, race/class bonuses, initialized state, no online metadata, and invalid selections.
- [x] 1.2 Initialize all generated character fields, progress bars, activity, equipment, inventory, spells, plot, quests, and simulation continuation so local simulation can advance a generated result; verify representative initial advancements succeed without mutating the original result.
- [x] 1.3 Add random name, race, class, and stats generation plus a stats-only reroll operation; verify Random replaces all selections and Reroll preserves the chosen name, race, and class.

## 2. Terminal New Guy flow

- [x] 2.1 Implement an alternate-screen interactive `new-guy` wizard with editable name, race/class selection, current stats preview, Random, Reroll, Sold!, and cancel controls; verify scripted terminal-event tests exercise each action and state transition.
- [x] 2.2 Reuse safe terminal setup and restoration handling for normal confirmation, cancellation, errors, and interrupts; verify cancellation restores the terminal and leaves the managed-character store unchanged.
- [x] 2.3 On Sold!, pass only the finalized generated character to the existing registration flow and display the established credential-safe managed identity; verify a confirmed wizard result creates exactly one readable managed character.

## 3. Flag-driven creation and boundaries

- [x] 3.1 Add `new-guy` CLI support where no explicit trait inputs start the wizard and complete `--name`, `--race`, and `--class` inputs create a character non-interactively; verify partial explicit input fails without registration and JSON output follows existing safe identity conventions.
- [x] 3.2 Add integration coverage that scripted creation registers an offline character, rejects invalid rule selections, and persists no online metadata or passkey.
- [x] 3.3 Add runtime-boundary regression tests proving offline generation and registration make no HTTP requests and do not construct or transmit leaderboard requests.

## 4. Validation

- [x] 4.1 Run formatting, warning-denied linting, generator/wizard/CLI tests, runtime-boundary tests, and the complete test suite; verify all checks pass.
- [x] 4.2 Replace global wizard hotkeys with navigable Name, Race, Class, and Stats rows whose focused row owns text-entry and action bindings; provide Name-only Random Name, manual race/class selection from randomized initial selections, and focused Stats Roll/Unroll actions; verify scripted events allow action-key characters in names and exercise each focus-specific action.
- [x] 4.3 Implement desktop-compatible base-stat rolls (`3 + 3d6`, each die in `0..5`), an unbounded provisional roll-history stack, and a pre-bonus total-quality indicator using dark gray (<46), gray (46–54), white (55–72), yellow (73–80), and red (>80); verify roll boundaries, Random Name scope, and multi-step unroll coverage.
- [x] 4.4 Run formatting, warning-denied linting, focused generator/wizard/CLI tests, runtime-boundary tests, and the complete test suite; verify all checks pass.
- [x] 4.5 Add shared name validation for a maximum of 30 Unicode scalar characters, at least one non-whitespace character, and no control characters; enforce it for generated, explicit CLI, and Sold! names while preserving invalid wizard drafts with a visible error; verify valid Unicode/internal-space, boundary-length, whitespace-only, empty, control-character, and overlong cases.
- [x] 4.6 Run formatting, warning-denied linting, focused generator/wizard/CLI tests, runtime-boundary tests, and the complete test suite; verify all checks pass.
- [x] 4.7 Seed a deterministic wizard-local PRNG from fresh OS randomness and store pre-roll PRNG state with roll history; make Unroll replay its restored roll so the following Roll reproduces the unrolled result; verify repeated Roll/Unroll/Roll sequences.
- [x] 4.8 Run formatting, warning-denied linting, focused wizard tests, runtime-boundary tests, and the complete test suite; verify all checks pass.
