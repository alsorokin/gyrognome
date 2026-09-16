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
  trait selection, Random Name, desktop-compatible stat roll quality, Roll,
  Unroll, Sold!, and cancellation.
- Support deterministic automation inputs without making flags the primary UX.
- Preserve existing credential redaction and no-network runtime guarantees.

**Non-Goals:**

- Browser-exact new-character random-call order or initial-state conformance
  fixtures beyond the desktop `3 + 3d6` stat roll and total indicator.
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

### Use focused controls with reversible, desktop-compatible stat rolls

The wizard tracks selected name, race, class, a current stats roll, and focus
on the Name, Race, Class, or Stats row. Navigation changes focus without
altering the provisional character. Name receives literal printable-key and
backspace edits only while focused, so letters used as actions elsewhere
(including `r` and `e`) remain typeable. Name exposes Random Name, Race and
Class expose manual cycling, and Stats exposes Roll and Unroll. Race and Class
start randomly selected but cannot be changed by an action other than their
focused manual controls. The footer displays the focused row's bindings plus
global Sold! and cancel controls.

Each stat roll stores six unmodified values, each calculated as `3 +` three
independent integers in `0..5`. The UI derives a pre-bonus sum from those
values and assigns its quality color using the desktop bands: dark gray
`<46`, gray `46..=54`, white `55..=72`, yellow `73..=80`, and red `>80`.
Race/class bonuses are applied only when producing the displayed canonical
character, not when calculating that indicator. A Roll pushes the current
base-stat values and the deterministic local PRNG state that preceded the
displayed roll onto an unbounded in-memory history before replacing them.
Unroll restores that state, regenerates the prior roll, and thereby positions
the PRNG so a subsequent Roll reproduces the result that was unrolled. Fresh
OS randomness seeds the deterministic PRNG once when the wizard opens. This
history is provisional and discarded on cancellation or registration.

This preserves a predictable stat preview, makes name entry unambiguous, and
allows every roll to be undone without persistence or network effects.

### Validate names at the generation boundary

The desktop form's `MaxLength = 30` establishes the compatibility length
limit, but its source contains no whitelist or confirmation validation. The
offline flow therefore accepts every Unicode scalar except control characters,
counts characters rather than bytes for the 30-character limit, and requires
at least one non-whitespace character. Internal Unicode whitespace remains
valid.

`generate` and every public generation entry point validate the selection name
before allocating a canonical character, giving scripts the same behavior as
the terminal UI. Sold! validates the provisional selection and retains the
wizard with a visible error on failure; it does not return a character to the
registration path. Name editing rejects additional characters once the limit
is reached and ignores control input, so an invalid draft can arise only from
an invalid externally supplied selection.

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
- [A context-specific key is interpreted globally] → Route events through the
  focused row and test focus changes, text entry, and each row's controls.
- [Bonus-modified stats obscure roll quality] → Preserve base dice values and
  calculate the indicator before applying selected trait bonuses.
- [An invalid name reaches persistence] → Validate at every generator entry
  point and block Sold! while retaining the interactive draft.
- [Unroll restores values but not the subsequent random sequence] → Store and
  replay the pre-roll deterministic PRNG state, with a Roll/Unroll/Roll test.
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
