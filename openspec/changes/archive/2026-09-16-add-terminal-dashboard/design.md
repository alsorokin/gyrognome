## Context

The local runtime already provides persisted, credential-safe canonical state
through `Store` and lifecycle operations through `Lifecycle`. See proposal.md
for motivation and `specs/terminal-dashboard/spec.md` for behavior. The
dashboard must observe these APIs without becoming a second runtime or exposing
the original browser document retained by the store.

## Goals / Non-Goals

**Goals:**

- Present one managed character as a responsive full-screen terminal view.
- Refresh durable state and runtime status safely while a worker may write.
- Route deliberate lifecycle actions through the existing lifecycle boundary.
- Always restore the caller's terminal state.

**Non-Goals:**

- Manage multiple characters from one dashboard session.
- Add progression controls, save export, data editing, or worker ownership.
- Recreate terminal state with a web UI or transmit state to another process.

## Decisions

### Use ratatui with crossterm behind a dashboard command

Add `gyrognome dashboard <character-id>` with a configurable refresh interval
whose default is one second. `ratatui` supplies layout and widgets while
`crossterm` supplies raw-mode, alternate-screen, and input events. The command
will validate the character before entering interactive mode.

This is the conventional Rust terminal UI pairing and permits a compact,
dependency-contained implementation. Handwritten ANSI rendering was rejected
because responsive layout, screen restoration, and input parsing would become
application responsibilities.

### Separate snapshot collection from rendering

A dashboard snapshot consists only of `ManagedCharacter` data and
`RuntimeStatus`, with a status message for the latest non-fatal action error.
A provider/controller interface obtains snapshots and runs lifecycle actions;
the renderer receives the snapshot rather than `Store`, raw documents, or a
database connection.

This makes redaction structural and enables deterministic tests for layouts,
keys, confirmation state, and error presentation with fakes. Direct database
access from widgets was rejected because it would make refresh errors,
concurrency, and testing harder to reason about.

### Poll with event timeouts and refresh on action completion

The event loop waits no longer than the configured refresh interval for input.
On timeout, explicit refresh, or lifecycle completion, it reloads the stored
character and service state. It never calls `Worker`, `simulation::advance`, or
lock acquisition. A read/write SQLite interaction is tolerated through the
existing store busy timeout.

Push notifications were rejected because the worker has no IPC channel and
adding one would exceed the dashboard's observer role. A one-second default
gives timely feedback while keeping a stopped runtime's dashboard idle.

### Use explicit key bindings and confirmation mode

The footer always shows `q` quit, `r` refresh, `s` start, `x` stop, and `c`
recover. Start, stop, and recover enter a confirmation prompt; `Enter`
confirms and `Esc` cancels. Lifecycle results replace the non-fatal status
message and trigger a refresh.

Confirmation avoids accidental service actions in a key-driven interface.
Unconfirmed one-key actions were rejected because stopping a worker is a
meaningful user operation.

### Guard terminal restoration with RAII

A terminal-session guard enables raw mode and the alternate screen only after
setup succeeds. Its `Drop` implementation disables raw mode and leaves the
alternate screen on every exit path; setup cleanup handles partial entry.
Lifecycle calls occur outside rendering but within the guarded session, so an
error returns through the same restoration path.

Manual cleanup at each return site was rejected because new error paths could
leave users in raw mode or an alternate screen.

## Risks / Trade-offs

- [The terminal is too small for every section] → Use stacked, scrollable or
  compact sections and show a minimum-size message rather than truncating
  secrets or crashing.
- [A user service manager is unavailable] → Preserve the last rendered state
  and render the lifecycle error in the status area.
- [SQLite is busy during a worker write] → Reuse the store's bounded busy
  timeout and retain the last successful snapshot if refresh fails.
- [Terminal cleanup itself fails] → Attempt all cleanup steps and return the
  original application error with cleanup context when possible.

## Migration Plan

1. Add terminal dependencies and presentation/controller modules without
   changing runtime storage, simulation, or lifecycle contracts.
2. Add the dashboard command and documentation for its key bindings.
3. Existing registered characters work immediately because the dashboard reads
   the current store schema; no data migration is required.
4. Rollback consists of exiting active dashboards and using the existing CLI
   commands; runtime data and user services remain unchanged.
