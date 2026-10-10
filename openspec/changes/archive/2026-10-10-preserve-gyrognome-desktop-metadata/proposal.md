# Proposal

## Why

Exporting a desktop character to `.pq` drops state that no desktop save
carries: tasks completed and elapsed time since import, the desktop random
continuation, import provenance, and the local-only advancement marker.
Re-importing the export into another Gyrognome instance resets all of it.
Losing the local-only marker also makes a locally advanced character look
like a fresh official-client import, which affects online reporting
eligibility.

## What Changes

- Desktop exports append a versioned Gyrognome metadata block after the last
  desktop component, inside the zlib payload. pq.exe 6.4.4 reads only its
  fixed component list, so it ignores the block on load and drops it on its
  next save.
- The block carries the since-import counters (tasks completed and elapsed
  milliseconds), the desktop random continuation, the provenance and
  adaptations, and the advancement provenance. It also carries a digest of
  the component stream it was written with.
- Import recognizes the block. A valid block restores those values instead of
  creating fresh registration metadata. Counters keep their "since import"
  labeling, now counted from the first import.
- If the digest doesn't match the components, import fails rather than
  falling back to a fresh import.
- Saves without the block import as they do today. Other unexplained trailing
  content is still rejected.
- **BREAKING (minor)**: Gyrognome builds without this change reject new
  exports as having unexplained trailing content.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `desktop-save-compatibility`: decoding accepts the Gyrognome metadata block
  as the only allowed trailing content; export writes the block; continuation
  provenance is restored from the block instead of reset.

## Impact

- `src/desktop_export.rs`: writes the block.
- `src/desktop_save.rs`: stops component parsing at the block, then bounds,
  verifies, and decodes it.
- `src/save.rs`, `src/runtime.rs` (`register_desktop`): carry the optional
  restored metadata and random state into registration.
- No new dependencies (`ring` and `serde_json` are already used).
- Unchanged: online eligibility rules, credential handling, and browser
  exports.
