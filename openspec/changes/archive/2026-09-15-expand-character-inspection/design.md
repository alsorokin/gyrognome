## Context

See proposal.md for motivation and the existing `pq-compatibility-core`
specification for compatibility requirements. The current importer validates only
top-level container shapes and projects identity plus realm into typed fields; the
original decoded JSON is retained for unmodified export.

## Goals / Non-Goals

**Goals:**

- Parse all simulation-relevant browser-save fields into an explicit, immutable
  canonical state representation.
- Make human-readable and JSON inspection useful for comparing browser save
  checkpoints.
- Preserve the current lossless raw-document export behavior.

**Non-Goals:**

- Mutating imported state, running the simulation, or persisting a character.
- Presenting the full raw browser document or any online passkey.
- Reproducing the browser's graphical UI.

## Decisions

### Use typed nested domain structures with ordered collections

Traits, bars, equipment, inventory, spells, plots, quests, task state, and online
metadata will have explicit types. Browser lists whose order is meaningful will
remain ordered rather than being converted to maps. Fixed-key fields will retain
their browser names for unambiguous comparison.

Using generic JSON traversal for the CLI was considered, but it would continue to
hide malformed nested state and make stable output difficult to test.

### Retain raw data beside, but separate from, canonical state

The importer will produce a typed canonical projection only after fully validating
the required browser fields, while retaining the source document as a private
round-trip payload. Inspection serializers will operate only on the canonical
projection, preventing accidental raw-field or passkey disclosure.

Serializing the raw document and filtering keys at output time was rejected because
new or nested secret-bearing data could bypass an incomplete filter.

### Provide text by default and canonical JSON by explicit format selection

`inspect <save>` will remain a concise terminal character sheet. A format option
will select JSON output for test automation and state comparisons. Both formats
will derive from the same canonical model and will have stable ordering.

Separate parsing paths for text and JSON were rejected because they could diverge
in field coverage or redaction behavior.

### Use synthetic fixtures and local disposable saves for different purposes

Committed fixtures will drive exact parser and output tests. Ignored disposable
saves may be manually inspected to compare real browser evolution, but will not be
read by automated tests or copied into the repository.

## Risks / Trade-offs

- [A browser update introduces new canonical fields] -> Retain unknown raw fields
  and report unsupported required shapes instead of silently guessing.
- [A complete text sheet becomes noisy] -> Group state into concise sections while
  leaving JSON available for exhaustive comparison.
- [Structured output leaks credentials] -> Do not include passkeys in canonical
  serializable state and add regression tests for both formats.
- [Browser field names and types are irregular] -> Capture those irregularities in
  fixture tests rather than normalizing them away.

## Migration Plan

1. Replace the shallow projection with typed nested state while preserving current
   successful-import behavior.
2. Add text and JSON inspection tests using synthetic fixtures.
3. Manually compare disposable browser saves with both formats.
4. Roll back by restoring the prior binary; imported files are never modified.
