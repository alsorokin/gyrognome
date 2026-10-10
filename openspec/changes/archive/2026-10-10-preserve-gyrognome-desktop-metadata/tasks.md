# Tasks

## 1. Metadata block

- [x] 1.1 Write the `GYROGNOME-META` block (version, component SHA-256, import metadata, random continuation) after the last component in `encode_desktop_save`, with the random state passed in from `export_save`. Verify with a unit test that the block follows the components and contains no credentials.
- [x] 1.2 Make `parse_components` stop at the magic, and add bounded block parsing that rejects bad length, oversize, unsupported version, a digest mismatch, and any bytes after the block. Verify with unit tests for each rejection and for saves without a block.

## 2. Import and registration

- [x] 2.1 Carry the optional restored metadata and random state through `import_supported_bytes` into `register_desktop`, and keep today's fresh values when there's no block. Verify with a runtime test where an advanced `LocalOnly` character with counters is exported and registered in a new store with identical counters, random state, provenance, and `LocalOnly`.
- [x] 2.2 Update the existing export round-trip tests and the desktop import inspection or registration messaging so restored counters are still labeled "since import". Verify that `cargo test desktop` passes.

## 3. Client check

- [x] 3.1 Open a fresh export in pq.exe 6.4.4 under Proton, let it save, then re-import that save. Verify that the client loads it without error and that the re-import has no block and fresh metadata. The user confirms the client visually.
