## Why

The local runtime now advances and persists managed characters, but observing
their state requires separate command invocations. A terminal dashboard makes
live local progression and service health understandable without exposing
browser credentials or duplicating runtime behavior.

## What Changes

- Add an interactive terminal dashboard for one managed character.
- Display credential-safe identity, activity, progress bars, equipment,
  inventory, spells, plots, quests, and runtime service status.
- Refresh persisted canonical state without taking character ownership or
  performing simulation advancement.
- Provide dashboard lifecycle actions that invoke the existing local runtime
  controls.
- Handle terminal restoration, unavailable characters, service failures, and
  unsupported simulation states with clear user-facing feedback.
- Keep browser save import/export, SQLite schema changes, worker scheduling,
  HTTP transport, and leaderboard reporting out of scope.

## Capabilities

### New Capabilities

- `terminal-dashboard`: Interactive, credential-safe terminal monitoring and
  lifecycle control for a locally managed character.

### Modified Capabilities

- None.

## Impact

- Adds terminal rendering and input dependencies to the Rust crate.
- Adds dashboard command, rendering, refresh, and action modules around the
  existing runtime read and lifecycle APIs.
- Uses local persisted state and user-service commands only; it does not
  introduce network communication or access raw browser save documents.
