# Design

## Context

pq.exe 6.4.4 `LoadGame` (pinned `Main.pas:1543-1574`) inflates the whole
file, calls `ReadComponent` once per form component (37 in total), and frees
the stream. It never looks at bytes after the last component. `SaveGame`
rewrites only those components. Today, Gyrognome's `parse_components` loops
until the end of the input, so any trailing bytes fail as a malformed
component. `register_desktop` always creates fresh `DesktopImportMetadata`
and a new random state.

## Goals / Non-Goals

**Goals:** Gyrognome-to-Gyrognome round trips keep the fields listed in the
spec; pq.exe still loads the export.

**Non-Goals:**
- Keeping the data after pq.exe saves.
- Preventing deliberate stripping of the block. Removing it already resets
  the character to a fresh import, the same as today.
- Carrying motto, guild, rested state, or credentials in the block. Motto,
  guild, and credentials are already in the components.

## Decisions

- **Placement: after the last component, inside the zlib payload.** This is
  the only spot pq.exe ignores on load and drops on save. Bytes after the
  zlib stream were rejected because ZLibEx's `ZDecompressStream` keeps
  feeding input after `Z_STREAM_END` and could fail or loop. New or unknown
  properties were rejected because they raise `EReadError`. Unused
  `Tag`/`Hint` slots were rejected because pq.exe would keep them while they
  go stale.
- **Layout:** ASCII magic `GYROGNOME-META` (it can't be confused with
  `TPF0`), a little-endian `u32` length, then a UTF-8 JSON object
  `{ "version": 1, "componentsSha256": "<hex>", "importMetadata": {...},
  "random": <u32> }`. JSON reuses the existing serde forms of
  `DesktopImportMetadata` and `DesktopRandomState`. The length is capped at
  64 KiB.
- **Digest:** SHA-256 (`ring`) over the inflated component bytes before the
  magic. This is exact, independent of version, and needs no canonical JSON.
  If it doesn't match, the import fails, as the user chose.
- **Parsing:** `parse_components` stops when the remaining bytes start with
  the magic. A separate function parses the block and requires that it ends
  exactly at the end of the payload. The decoded save gets an optional
  `restored: Option<(DesktopImportMetadata, DesktopRandomState)>` field.
- **Registration:** `register_desktop` uses the restored metadata and random
  state when present, and otherwise keeps today's behavior.
- **Validation:** the block stores only a desktop random state, so a
  browser continuation cannot be restored. Adaptations come from
  the block, not from re-detection, because the exported components are
  already normalized.

## Risks / Trade-offs

- [Older Gyrognome builds reject new exports] -> Accept it; it's a small
  personal tool. Mention it in the CHANGELOG.
- [Anyone can strip the block to clear `LocalOnly`] -> The same is true of a
  pq.exe round trip today. Stripping isn't treated as a threat.
- [A pq.exe build could behave differently from the pinned source] -> One
  manual Proton smoke test with a real export.
