## 1. Desktop writer

- [x] 1.1 Study pq6 Main.dfm/Main.pas and fixtures; list components/properties to emit
- [x] 1.2 Implement Delphi component-stream + zlib encoder from canonical state and private metadata
- [x] 1.3 Round-trip tests (encode -> import -> equal state), synthetic credentials only

## 2. Export command

- [x] 2.1 Browser export from stored document overlaid with current canonical state
- [x] 2.2 Desktop export via the new encoder with stored credentials
- [x] 2.3 CLI `export <id> [-o PATH] [--force]` with default name, overwrite prompt, atomic 0600 write
- [x] 2.4 Stop/export/restart of a running worker, restarting on failure

## 3. Verification

- [x] 3.1 Tests for overwrite prompt/force, running-worker handling, browser round trip
- [x] 3.2 Open an exported `.pq` in pq.exe under Proton and compare the character; update README/docs
