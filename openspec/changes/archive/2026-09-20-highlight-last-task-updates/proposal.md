# Proposal

## Why

The dashboard currently refreshes canonical character state but gives no visual
indication of what a completed task changed. Operators must compare spell,
inventory, stat, and equipment values manually, unlike the official client,
which retains selections for values changed by the most recently completed
task.

## What Changes

- Highlight dashboard spell, inventory-item, stat, and equipment values that
  changed while processing the most recently completed task.
- Replace prior highlights on the next completed task, preserving them during
  ordinary in-progress refreshes.
- Apply the indicator in both full and compact dashboard layouts without
  changing canonical state, simulation behavior, persistence, or dashboard
  side effects.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `terminal-dashboard`: Visually identify character list values updated by the
  most recently completed task in the live terminal dashboard.

## Impact

- Affects dashboard snapshot comparison, stateful refresh handling, and
  Ratatui rendering in `src/dashboard.rs`.
- Adds focused dashboard rendering and refresh-state tests.
- Does not add dependencies, alter the CLI, write character state, or change
  the simulation/runtime boundary.
