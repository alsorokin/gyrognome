## Why

Gyrognome can import and locally advance an existing browser character, but it
cannot create a new offline character. A native creation flow lets users begin
locally while preserving the familiar Progress Quest "New Guy" experience.

## What Changes

- Add an offline-only character-generation capability that creates valid local
  characters without loading a browser save, contacting a server, or creating
  online metadata.
- Add an interactive terminal "New Guy" wizard as the default creation
  experience: choose or randomize name, race, and class; reroll stats; confirm
  with "Sold!"; or cancel without persisting a character.
- Add a flag-driven, non-interactive creation mode for scripting and tests.
- Register confirmed generated characters through the existing managed-local
  character store.
- Use fresh local randomness for generation; browser-exact character-creation
  RNG behavior and leaderboard reporting are out of scope.

## Capabilities

### New Capabilities

- `offline-newguy`: Creates and registers offline-only Progress Quest
  characters through an interactive wizard or explicit CLI inputs.

### Modified Capabilities

- None.

## Impact

- Adds character-generation and terminal-wizard modules plus a `new-guy` CLI
  command.
- Uses the bundled ruleset, canonical character types, local store, and
  existing terminal dependencies.
- Does not add HTTP transport, live passkey handling, online character
  creation, leaderboard reporting, or browser-conformance fixtures.
