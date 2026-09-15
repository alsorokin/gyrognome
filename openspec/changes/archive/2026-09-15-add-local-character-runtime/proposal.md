## Why

Gyrognome can inspect and deterministically advance a character in memory, but
cannot retain or safely run that character between invocations. A local runtime
is needed to make offline progression durable and prevent two processes from
advancing the same character concurrently.

## What Changes

- Add a local-only character runtime that imports a browser save into a
  per-user SQLite data store and persists canonical state after simulation
  advancement.
- Add process ownership protection so at most one runtime instance can manage a
  character at a time, including clear handling of stale ownership after an
  unexpected exit.
- Add command-line lifecycle operations to create, start, inspect, stop, and
  recover a locally managed character runtime.
- Run advancement from elapsed wall-clock time only at the runtime boundary;
  continue to invoke the simulation core with explicit durations.
- Keep browser-save export, terminal dashboard rendering, HTTP transport, and
  leaderboard reporting out of scope.

## Capabilities

### New Capabilities

- `local-character-runtime`: Persist locally managed characters, exclusively
  own their runtime processes, and provide lifecycle-controlled offline
  advancement.

### Modified Capabilities

- None.

## Impact

- Adds SQLite storage and local process/service-management dependencies to the
  Rust crate.
- Adds runtime, persistence, locking, and lifecycle CLI modules around the
  existing pure simulation API.
- Introduces local state files under the invoking user's data directory; neither
  runtime execution nor persisted data enables network communication.
