# Design

## Context

See proposal.md. Import is data-only (`desktop_save.rs`: `parse_components`,
`map_desktop_document`, `validate_desktop_save`). Desktop characters persist
`DesktopCanonicalState` plus `desktop_private` credentials (schema v4, 0600);
browser characters keep `original_document` and `save::export` already
produces `.pqw`. The pq6 source (bitbucket.org/grumdrig/pq, pq6 branch;
Main.dfm/Main.pas) defines the component layout and load semantics.

## Goals / Non-Goals

**Goals:** export both profiles; round-trip identical state; real pq.exe opens
the result.

**Non-Goals:** dashboard UI; desktop-to-browser conversion; byte-identical
reproduction of original saves.

## Decisions

- **Template-based `.pq` writer.** Emit the fixed component tree the importer
  expects (ExpBar, EncumBar, PlotBar, QuestBar, TaskBar, Traits, Stats,
  Equips, Inventory, Spells, Plots, Quests, fQuest, game-style label, etc.) in
  Delphi stream form, zlib-compressed. Property names, order and types come
  from the pq6 DFM and real fixtures. Alternative (generic serializer of
  `DesktopDocument`) rejected: the parsed document is lossy by design.
- **Pure encoder over `DesktopCanonicalState` + private metadata**, returning
  bytes; validated by parsing its own output via the existing importer.
- **Browser export** overlays the current canonical state on the stored
  document (as the runtime test at `runtime.rs` does) before `save::export`.
- **Stop/export/restart** via existing lifecycle worker control; restart in a
  guard that runs on failure. Simplest option, avoids snapshotting a live
  worker.
- **Output:** `export <id> [-o PATH] [--force]`; prompt on TTY only, refuse
  without TTY and without `--force`; temp file in the target directory, 0600,
  rename into place.
- **Verification:** Rust round-trip tests with synthetic credentials, plus a
  smoke test in pq.exe (`/home/snay/Games/pq6-4-4/pq.exe`) under Proton; the
  user confirms visually if headless checking is impractical.

## Risks / Trade-offs

- [pq.exe rejects a subtly wrong stream] -> follow pq6 source and fixtures;
  Proton smoke test.
- [Adaptations (legacy prologue, spelling patch, quest placeholder) write
  normalized form] -> confirm client accepts it in the smoke test.
- [Credentials in exports] -> 0600, never printed, synthetic data in tests.
- [Restart after failure leaves worker in unexpected state] -> restart only if
  it was running before.
