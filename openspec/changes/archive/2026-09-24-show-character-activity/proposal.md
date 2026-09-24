## Why

The dashboard's character-selection screen currently identifies each registered character but does not help users choose the most recently used or currently running one. Showing access recency and active runtime state at selection time reduces guesswork before entering the full dashboard.

## What Changes

- Update the dashboard character-selection flow to order registered characters by last access time descending.
- Display each character's last accessed time in the selector metadata.
- Display whether each character is currently active in the selector metadata.
- Preserve credential-safe behavior: do not expose browser passkeys, raw save documents, raw endpoint bodies, or unrecognized raw save fields.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `terminal-dashboard`: Extend dashboard selection behavior to include access recency, active runtime state, and last-access-first ordering.

## Impact

- Affected code: `src/cli.rs`, `src/dashboard.rs`, `src/runtime.rs`, `src/lifecycle.rs` if shared status/access models are needed, and dashboard selector tests in `src/dashboard.rs` plus CLI/runtime integration tests as appropriate.
- APIs/data: Internal selector inputs need to carry last-access metadata and runtime state; persisted storage already exposes `updated_at_unix_ms` on managed characters, but list ordering currently uses creation time.
- Dependencies: No new external dependencies expected.
- Systems: Selector active-state checks should use existing local systemd/service status integration and remain local-only without network requests.
