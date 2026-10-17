# Design

## Context

See `proposal.md` for motivation and the delta specification for the behavior
contract. `Command::Dashboard` in `src/cli.rs` currently accepts an optional
ID and `refresh_ms`; without an ID it builds selector entries and enters the
selector. It validates the selected character through `LocalProvider::refresh`
before calling `dashboard::run`.

`selector_entries` in `src/dashboard.rs` labels each character's
`updated_at_unix_ms` as last accessed and sorts descending, with stable-ID
tie-breaking. Reading a character does not record an access. Worker commits
and profile writes update that timestamp, so the existing ordering cannot
implement dashboard-only recency without changing its writers.

`Store` owns the per-user SQLite database under the existing XDG data root.
Its versioned schema initialization already uses transactions and creates
backups before upgrading existing databases. `LocalProvider` wraps a `Store`
in a `RefCell`; the dashboard restores terminal state through `TerminalSession`.
Current dashboard tests cover selector ordering and pre-terminal errors.

## Goals / Non-Goals

**Goals:** Use one per-character last-access timestamp for both selector
presentation and resume; reuse the existing persisted timestamp, selection,
validation, storage errors, and terminal cleanup.

**Non-Goals:** Do not introduce a dedicated resume property, preference table,
or full access log. Do not remember layout preferences or change simulation,
worker lifecycle, online actions, or character save formats.

## Decisions

### Add an explicit, conflicting CLI flag

Add a boolean `resume` with Clap long and short forms and a conflict with `id`.
Treat the conflict as a small interface assumption: rejecting contradictory
target selection is clearer than silently ignoring an explicit ID or flag.
Keep `--refresh-ms` orthogonal. Plain no-ID invocation always uses the selector,
even when previously opened characters exist.

Resolve resume from currently registered character summaries using the same
ordering as the selector: last-access time descending, with stable identifier
ascending for ties. Share the ordering logic rather than duplicating subtly
different comparators. Open the first entry if it has a recorded access;
otherwise run the existing selector, including its empty-store error and
cancellation behavior. A deleted character is naturally absent, so the next
most recently opened registered character wins. No stale-ID property needs
cleanup. Propagate storage errors rather than hiding them as empty history.

### Repurpose the existing persisted timestamp

Use the existing `characters.updated_at_unix_ms` column as dashboard-access
metadata instead of adding another persisted field. Keep its legacy storage
and serialized field name to avoid unrelated API renames, but document the new
meaning in the runtime types. Add a narrowly named `Store` method to record a
dashboard opening. Retain `SelectorEntry.last_accessed_unix_ms` as the
presentation name.

Initialize the value to zero on browser and desktop registration, denoting
never opened. Show that sentinel as `Never` rather than a 1970 date. A
successful dashboard opening writes the current Unix millisecond time for
that character only. Equal times use the existing stable-ID tie-breaker;
no separate sequence or history is needed.

Remove timestamp assignments from browser and desktop worker persistence,
profile changes, and any other non-dashboard write paths, including returned
in-memory records. Preserve all actual state writes and rested/worker timing
updates. Worker writes must not overwrite an access time recorded by a
concurrent dashboard.

Increment the database schema version and reset this column to zero in the
existing migration transaction for older stores. Historical values represent
worker/profile updates, not known openings, so retaining them would fabricate
access history. This is a one-time semantic migration, not an added table.
Reuse current migration backups and database permissions.

### Record once after the first successful dashboard draw

Add a narrowly scoped opening notification to the dashboard provider interface,
implemented by `LocalProvider` using the access-time write method. Invoke it once
from `dashboard::run`, immediately after the initial snapshot has been rendered
successfully, before the normal event loop continues. Test providers implement
the notification without filesystem access.

Do not put this write in `refresh`: refresh is also called during preflight
validation and repeatedly while observing the character. Recording after the
first draw avoids recording access on failed validation, terminal setup,
initial state collection, or initial rendering. A later session error does not
undo a successfully recorded access. Propagate an access-time write error through the
existing dashboard error path so the terminal guard restores the screen before
the CLI reports it; do not silently claim persistence succeeded.

This updates UI-access metadata only. Existing read-only canonical-state and
refresh guarantees remain intact. Newly read selector entries immediately
reflect the same timestamps that resume uses.

## Risks / Trade-offs

- [Legacy update times cannot recover actual dashboard history] -> Reset them
  to never opened once; document that resume uses the selector until a
  dashboard has been opened after upgrade.
- [Non-dashboard writers could keep reordering the list] -> Cover browser and
  desktop worker writes and profile edits in focused regression tests,
  including a worker record loaded before a dashboard access.
- [Equal millisecond timestamps or wall-clock adjustments] -> Share the
  deterministic timestamp/ID ordering with the selector. Do not add a second
  ordering property or promise chronology beyond the recorded access times.
- [The legacy `updated_at_unix_ms` name is less descriptive] -> Retain it as a
  bounded compatibility shortcut and document dashboard-only semantics; do not
  expose it as a general saved-update time.
- [The target can be deleted between lookup and rendering] -> Resolve from the
  current registered list; retain the ordinary actionable not-found error for
  a later deletion rather than adding a retry loop.

## Migration Plan

Reset legacy `updated_at_unix_ms` values to the never-opened sentinel through
the next schema migration, without changing canonical state, profile values,
rested accounting, or worker timing baselines. Fresh registrations use the same
sentinel. The first successful dashboard opening sets a real access timestamp.
No dedicated resume storage is created.

As with current migrations, an older binary rejects a newer schema. If rollback
is necessary, stop affected processes and restore the pre-upgrade database
backup using the established backup location; reverting the binary alone is
not a schema downgrade. Restoring that backup also discards subsequent writes.
