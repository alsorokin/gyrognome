## Context

Gyrognome already has bundled character rules, canonical state types, an
offline simulation engine, a managed-character SQLite store, and terminal UI
dependencies. `Store::register` accepts an in-memory canonical character, so
the creation flow can reuse the registration and runtime path used by imported
saves. See proposal.md for motivation and specs/offline-newguy/spec.md for
behavioral requirements.

## Goals / Non-Goals

**Goals:**

- Produce internally valid, level-one, offline-only canonical characters.
- Make the default terminal flow recognizable as Progress Quest's New Guy:
  trait selection, Random, stat-only Reroll, Sold!, and cancellation.
- Support deterministic automation inputs without making flags the primary UX.
- Preserve existing credential redaction and no-network runtime guarantees.

**Non-Goals:**

- Browser-exact new-character RNG, random-call order, stat ranges, or initial
  state conformance fixtures.
- Online character creation, passkeys, reports, or leaderboard transport.
- Changing imported-save handling, simulation behavior, or database schema.

## Decisions

### Isolate generation from UI and persistence

Create a pure generator that accepts explicit traits plus a local randomness
source and returns a canonical `Character`. The terminal wizard holds a
provisional generated result and calls the existing store registration API only
after Sold!.

This keeps cancellation side-effect-free and permits direct unit tests. It is
preferred over having the UI mutate stored state because no draft records,
cleanup logic, or partial-character recovery are needed.

### Make the terminal wizard the no-input default

`gyrognome new-guy` starts an alternate-screen terminal wizard when all
creation traits are omitted. Explicit `--name`, `--race`, and `--class` form
the scripted path; the command rejects partial trait input rather than mixing
prompts with automation.

This makes the normal experience interactive while retaining an unambiguous
automation contract. A line-prompt interaction was considered but rejected
because the existing terminal dashboard establishes terminal UI support and
the New Guy flow needs in-place randomization, rerolls, and preview.

### Preserve Random and Reroll as distinct operations

The wizard tracks selected name, race, class, and a current stats roll.
Random replaces all four; Reroll replaces only the stats roll. Selection
changes retain the displayed stats until the user explicitly rerolls or
randomizes.

This directly reflects the agreed browser-like user experience and makes the
visible state predictable. Treating either action as a complete replacement
would blur their user-visible distinction.

### Create a minimal valid offline initial state

The generator selects only from the bundled tables, applies the selected race
and class bonuses, initializes all required canonical fields and progress bars,
and sets `online` to absent. Fresh OS-provided local randomness seeds initial
selection and the simulation continuation.

This satisfies local simulation and persistence contracts without falsely
claiming browser creation conformance. Replaying browser creation algorithms
is deferred until online creation/reporting needs establish that as a
requirement.

### Keep registration and output credential-safe

The command reuses `Store::register` and the existing managed-character
identity/output conventions. The generated character never has an original
browser save or online credential, and JSON output stays restricted to safe
identity data.

## Risks / Trade-offs

- [A superficially valid initial state fails later simulation] → Define
  generator invariants from the canonical type and validate generated
  characters by advancing representative initial intervals in tests.
- [Terminal errors leave the user's screen unusable] → Use the dashboard's
  terminal setup/restoration pattern for normal exit, errors, and interrupts.
- [OS randomness fails or cannot be read] → Return a clear creation failure
  before registration, leaving no persisted character.
- [A future online feature assumes a browser-created character] → Keep
  offline-generated characters explicitly without `online` metadata; a future
  online change must state its migration or eligibility rules.

## Migration Plan

The change adds a new command and creates standard managed-character records,
so no data migration is required. Removing the feature only requires stopping
new creation; existing generated characters remain ordinary local managed
characters and can continue under the existing runtime.
