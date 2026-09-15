## Context

The current crate imports browser saves, holds complete canonical character
state in memory, and offers a pure simulation API that accepts a supplied
elapsed duration. See proposal.md for motivation and
`specs/local-character-runtime/spec.md` for externally observable behavior.
The next layer must own persistence, clocks, process lifetime, and operating
system integration without weakening the simulation module's purity or the
project's offline-only boundary.

## Goals / Non-Goals

**Goals:**

- Make imported characters durable and individually addressable on one Linux
  user account.
- Drive deterministic simulation from a runtime-owned clock with atomic state
  persistence.
- Provide reliable exclusive ownership and `systemd --user` lifecycle control.
- Preserve sufficient original save data for a future browser-compatible export
  without exposing credentials through normal diagnostics.

**Non-Goals:**

- Catch up progression for time spent while a service is stopped.
- Send reports, create online characters, or add any HTTP dependency.
- Render a terminal dashboard or support cross-machine synchronization.
- Turn the pure simulation API into an I/O-aware interface.

## Decisions

### Use one SQLite database in the XDG user data directory

The runtime will resolve its data root from `XDG_DATA_HOME`, falling back to
`~/.local/share`, and store a single Gyrognome SQLite database there. A
`characters` record will contain a generated stable identifier, credential-safe
identity fields for listing, a versioned serialized canonical state, the
original imported save document, and timestamps/runtime metadata. Simulation
results and their associated metadata will be committed in one transaction.

SQLite provides durable, inspectable, per-user storage with transactional
crash recovery and avoids operating a separate database service. A collection
of JSON files was rejected because it cannot provide equivalent atomic state
and metadata updates or safe enumeration as the character set grows.

The persisted original document is retained for future export work; updated
canonical state is persisted separately until that future change defines how to
merge it back into an export document without losing unknown browser fields.

### Keep the runtime as an adapter around pure simulation

The runtime worker owns a monotonic elapsed-time source and periodically
converts each active interval to milliseconds. It loads the last committed
canonical state, calls `simulation::advance(state, ruleset, elapsed_ms)`, and
atomically commits the returned state only after a successful result.

After a worker starts, its baseline is the current monotonic instant. A restart
does not calculate duration from a persisted wall-clock timestamp, so downtime
does not advance a character. This deliberately favors user-controlled,
observable execution over offline catch-up and avoids treating clock changes as
game time. Passing wall-clock time directly into the simulation core was
rejected because it would violate the existing deterministic-simulation
contract and make conformance testing harder.

### Enforce exclusive ownership with a per-character advisory file lock

Before a worker reads or advances a character, it takes a non-blocking
exclusive operating-system lock for that character under the same data root and
keeps the lock handle for its full lifetime. The lock is automatically released
when the process ends, including a crash, while the database remains the
durable source of truth.

SQLite write locking alone was rejected because it serializes writes but does
not express exclusive long-lived runtime ownership. PID files were rejected
because stale files require ambiguous liveness checks and manual cleanup.

### Model lifecycle as a systemd user template service

Install a `gyrognome@<character-id>.service` user unit whose worker command
runs one managed character. CLI lifecycle subcommands validate the character
locally and delegate start, stop, and status operations to `systemctl --user`.
The worker itself always acquires the advisory lock, so a direct or duplicate
launch remains safe even if service-manager state is stale.

Using a user-scoped service permits startup, stopping, logging, and recovery to
follow standard Linux service semantics without root access. An application
daemon with ad hoc PID and signal management was rejected because it would
duplicate service-management behavior and widen the failure surface.

### Make failure visible and preserve the last good state

The worker will terminate non-successfully when simulation yields an unsupported
transition or persistence fails. It will not skip the failed duration, invent a
result, or overwrite the stored state. Lifecycle status will distinguish an
inactive service, an active owner, and a failed worker where the service manager
can provide the operational detail.

## Risks / Trade-offs

- [A machine or container lacks a usable user service manager] → Lifecycle
  commands detect and explain this prerequisite; persisted characters remain
  intact and can be inspected.
- [A crash occurs between simulation and persistence] → Compute the next state
  in memory and commit it atomically only after simulation succeeds; on restart
  use the last committed state.
- [The service is started twice through different paths] → The runtime's
  non-blocking per-character lock rejects the duplicate worker independent of
  `systemd` unit state.
- [A future export needs browser unknown fields after progression] → Retain the
  original document and version canonical serialization now; defer merge and
  export semantics to a dedicated save-mutation change.
- [Runtime intervals are delayed by process scheduling] → Measure elapsed time
  from the active worker's monotonic clock and rely on the simulation core's
  bounded tick behavior.

## Migration Plan

1. Add storage, locking, worker, and lifecycle modules behind new CLI commands;
   existing `inspect` and library simulation calls remain unchanged.
2. Install or document the user service template during local runtime setup.
3. Register characters explicitly from browser saves; no existing `.pqw` input
   files or in-memory state are migrated automatically.
4. On rollback, stop any Gyrognome user services before removing the new binary
   or data directory. Retained SQLite data and original documents remain
   recoverable for a future compatible version.
