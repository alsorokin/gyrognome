# Proposal

## Why

Gyrognome can create, advance, inspect, and report online characters, but it
still lacks the official client's remaining online profile actions: changing a
character motto and setting its guild designation, with an empty designation
leaving the current guild. Completing those actions also provides an
opportunity to make the dashboard's limited vertical space more useful and its
elapsed-time display easier to read.

## What Changes

- Add persistent, credential-safe online profile metadata for a managed
  character's motto and guild designation, including optional values imported
  from browser saves.
- Add CLI and dashboard actions for changing or clearing a motto and submitting
  a guild designation, including an empty designation to leave, while the
  character runtime is active.
- Include the persisted motto in subsequent manual and automatic leaderboard
  reports.
- Serialize foreground online profile actions with worker-generated reports so
  their request ordering and profile snapshots are well defined without
  surrendering runtime ownership.
- Extend the disposable browser conformance procedure and evidence gate to
  cover non-empty and empty guild-designation submissions before production
  guild requests are enabled.
- Replace the dashboard's misleading Quest target line with conditional Motto
  and Guild lines when those values are present; reduce the Activity pane by
  one row; cap Equipment content at the eleven supported equipment slots; and
  render Last task elapsed as days, hours, minutes, and seconds rather than a
  raw second count.
- Update user documentation for the new commands, dashboard keys, online
  eligibility, outcomes, and conformance requirements.

## Capabilities

### New Capabilities

- `online-character-profile`: Persistent motto and guild profile state plus
  explicit CLI and dashboard actions that synchronize it with the official
  endpoint.

### Modified Capabilities

- `pq-compatibility-core`: Browser save interchange includes optional motto and
  guild fields as typed, credential-safe state.
- `local-character-runtime`: Managed persistence and active workers preserve
  online profile metadata and use the current motto in automatic reports.
- `opt-in-leaderboard-reporting`: Explicit motto and guild operations join the
  gated, official-endpoint-only reporting surface with safe outcomes and no
  automatic retry.
- `leaderboard-conformance`: Disposable browser evidence covers non-empty and
  empty guild-designation requests and sanitized endpoint outcomes.
- `terminal-dashboard`: The dashboard exposes live motto and guild editing and
  revises pane contents, sizing, and elapsed-time formatting.

## Impact

The change affects browser-save parsing, managed-character SQLite schema and
migration, worker/report synchronization, reporting transport response
handling, CLI commands, dashboard state/input/rendering, conformance scripts
and fixtures, integration tests, and README documentation. It adds no new
external service or dependency and continues to restrict authenticated traffic
to the official Progress Quest endpoint.
