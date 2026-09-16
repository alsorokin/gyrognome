## 1. Offline character generation

- [ ] 1.1 Add a pure offline character generator that validates bundled-ruleset race and class selections, produces valid level-one canonical state from fresh local randomness, and verify generator unit tests cover traits, race/class bonuses, initialized state, no online metadata, and invalid selections.
- [ ] 1.2 Initialize all generated character fields, progress bars, activity, equipment, inventory, spells, plot, quests, and simulation continuation so local simulation can advance a generated result; verify representative initial advancements succeed without mutating the original result.
- [ ] 1.3 Add random name, race, class, and stats generation plus a stats-only reroll operation; verify Random replaces all selections and Reroll preserves the chosen name, race, and class.

## 2. Terminal New Guy flow

- [ ] 2.1 Implement an alternate-screen interactive `new-guy` wizard with editable name, race/class selection, current stats preview, Random, Reroll, Sold!, and cancel controls; verify scripted terminal-event tests exercise each action and state transition.
- [ ] 2.2 Reuse safe terminal setup and restoration handling for normal confirmation, cancellation, errors, and interrupts; verify cancellation restores the terminal and leaves the managed-character store unchanged.
- [ ] 2.3 On Sold!, pass only the finalized generated character to the existing registration flow and display the established credential-safe managed identity; verify a confirmed wizard result creates exactly one readable managed character.

## 3. Flag-driven creation and boundaries

- [ ] 3.1 Add `new-guy` CLI support where no explicit trait inputs start the wizard and complete `--name`, `--race`, and `--class` inputs create a character non-interactively; verify partial explicit input fails without registration and JSON output follows existing safe identity conventions.
- [ ] 3.2 Add integration coverage that scripted creation registers an offline character, rejects invalid rule selections, and persists no online metadata or passkey.
- [ ] 3.3 Add runtime-boundary regression tests proving offline generation and registration make no HTTP requests and do not construct or transmit leaderboard requests.

## 4. Validation

- [ ] 4.1 Run formatting, warning-denied linting, generator/wizard/CLI tests, runtime-boundary tests, and the complete test suite; verify all checks pass.
