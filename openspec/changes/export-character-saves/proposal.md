# Proposal

## Why

Gyrognome can import `.pqw` and `.pq` saves, but registered characters live only
in the local SQLite store. A lost or corrupted database loses them, and they
cannot be opened in the browser or desktop client again.

## What Changes

- Add `gyrognome export <id> [-o PATH] [--force]`.
- Browser characters export to `.pqw`; desktop characters export to `.pq`.
- Default output is `./<save-name>.pqw` or `./<save-name>.pq`; overwriting an
  existing file prompts unless `--force` is given. Writes are atomic with mode
  `0600` because exports contain credentials.
- Exporting a running character stops its worker, snapshots, writes the file,
  then restarts the worker if it was running (also on export failure).
- Add a Delphi component-stream + zlib `.pq` writer that emits the layout the
  existing importer and the real 6.4.4 client accept. Dropped data is limited to
  non-save state (import counters, provenance, RNG continuation history).
- Desktop passkey/account/password are already persisted in `desktop_private`;
  browser credentials come from the stored original document. No schema change
  is expected.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `desktop-save-compatibility`: add `.pq` export that round-trips to identical
  canonical state and is loadable by pq.exe 6.4.4.
- `local-character-runtime`: add the export command, overwrite/force behavior,
  secure file writing, and stop/export/restart handling of live characters.

## Impact

- Code: `src/cli.rs`, `src/save.rs`, `src/desktop_save.rs` (writer), `src/runtime.rs`
  (snapshot and credential access), lifecycle/worker control.
- Tests: Rust round-trip tests with synthetic credentials; a manual or
  Proton-run smoke test of an exported `.pq` in pq.exe.
- Non-goals: dashboard UI, desktop-to-browser conversion, byte-identical output
  with original desktop saves.
- Risk: pq.exe strictness; mitigated by following the pq6 source and fixtures.
