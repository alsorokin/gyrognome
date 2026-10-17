# Proposal

## Why

`gyro dashboard --resume` (or `-r`) should reopen the character the user last viewed without requiring a selection or remembering an ID. The selector's "last accessed" time should mean last opened in the dashboard, not last updated by a worker.

## What Changes

- Add equivalent `--resume` and `-r` flags to `gyro dashboard`, mutually exclusive with an explicit character ID.
- Update each character's existing persisted last-access timestamp only after a successful dashboard opening, whether by ID, selection, or resume; worker and profile updates no longer change it.
- Use the same dashboard-access ordering for the selector and resume, without a separate remembered-character property or preference table.
- Resume the most recently opened registered character immediately. If that character was deleted, resume the next most recently opened character; if no previously opened characters remain, show the normal selector.
- Display never-opened characters after opened characters with a "Never" access time. Reset legacy worker-derived timestamps during migration rather than presenting them as dashboard accesses.
- Keep plain `gyro dashboard` interactive and preserve character progression and service behavior.
- Document the corrected recency meaning, flags, and fallback behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `terminal-dashboard`: Define last accessed as the last successful dashboard opening and use that shared recency for selection and opt-in resumption.

## Impact

Touches CLI parsing and routing in `src/cli.rs`, dashboard startup and selector presentation in `src/dashboard.rs`, timestamp initialization, writers, and migration in `src/runtime.rs`, focused tests, and dashboard usage in `README.md`. Reuses the existing timestamp rather than adding resume storage; its meaning changes from saved-update time to dashboard-access time. No new dependencies or network operations are required. Canonical state, profile values, rested accounting, and worker timing baselines must remain unchanged when recording a dashboard opening.
