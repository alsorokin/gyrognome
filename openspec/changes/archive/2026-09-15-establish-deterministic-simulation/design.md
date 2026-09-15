## Context

See proposal.md for motivation. The current compatibility core parses complete
browser saves and exposes Alea state twice: once in character state and once in
the runtime RNG module. Browser `main.js` uses a capped 100 ms timer increment
and applies nearly all game effects only after task completion. Its outcomes use
the `K.*` rule tables.

## Goals / Non-Goals

**Goals:**

- Establish one canonical Alea state representation and a pure state transition
  API.
- Reproduce browser task-completion behavior in a traceable, fixture-driven way.
- Make the browser ruleset an explicit, revisioned simulation input.

**Non-Goals:**

- A real-time clock loop, process lifecycle, persistence, save export mutation,
  UI updates, or online reporting.
- Guessing at game rules from the visible state instead of porting browser
  behavior and data.

## Decisions

### Simulate supplied elapsed time, not wall-clock time

The engine will accept elapsed milliseconds from its caller, split advancement
into browser-compatible bounded timer increments, and only process completion
effects after a task reaches its duration. This makes simulation reproducible in
tests and lets a later runtime own sleep, pause, and catch-up policy.

Embedding a timer in the engine was rejected because it would make checkpoint
tests nondeterministic and would decide lifecycle behavior prematurely.

### Use immutable input and explicit resulting state

Simulation will consume a canonical state plus a selected ruleset and return a
new state or a field-specific simulation error. It will not update the retained
raw save document; save-editing/export policy remains a later change.

In-place mutation was rejected because it obscures comparisons with browser
checkpoints and risks accidentally treating inspection data as persisted state.

### Consolidate Alea state at the simulation boundary

Character-state Alea values and the RNG continuation type will become one shared
type. It will preserve JavaScript `Number` behavior, including fractional
results from `uint32()` and modulo operations, rather than coercing values to
Rust integers.

Integer-only random helpers were rejected because they would shift later RNG
consumption and break browser equivalence.

### Bundle an explicit browser ruleset

The ruleset will be represented as structured, ordered data with a source URL
and revision recorded alongside it. Simulation receives a ruleset identifier or
reference explicitly, so future ruleset revisions cannot silently change an
existing character's outcome.

Fetching tables dynamically was rejected because it prevents offline
reproducibility and makes a checkpoint depend on mutable remote data.

### Build around golden conformance checkpoints

Fixtures will record a sanitized initial state, a finite sequence of elapsed-time
inputs, and the expected full canonical state/RNG result. Tests compare complete
serializable canonical output, not just user-visible summaries.

Unit tests of individual reward paths alone were rejected because they miss
random-call ordering errors that accumulate across a real completion.

## Risks / Trade-offs

- [The browser ruleset is large and irregular] -> Import it as source-versioned
  data first, preserving order and textual spellings before refactoring.
- [A save contains UI-derived cached display fields] -> Treat browser equations
  and canonical state values as authoritative; recompute derived fields only
  when their semantics are observed and covered by checkpoints.
- [Floating-point differences affect bar state] -> Use `f64` with operation
  order matching browser JavaScript and assert exact serialized checkpoint values.
- [Browser behavior is under-observed] -> Add disposable-session checkpoints
  before porting each task-completion family.

## Migration Plan

1. Introduce the pure engine and shared Alea representation alongside the
   read-only importer.
2. Add ruleset data and checkpoints before enabling each behavior family.
3. Keep current `inspect` behavior unchanged while simulation remains
   library-only.
4. Roll back by removing the new simulation API; existing saved documents remain
   untouched because this change performs no writes.
