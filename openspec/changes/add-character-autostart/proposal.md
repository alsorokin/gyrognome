# Proposal

## Why

Characters currently require a manual start after each computer restart. A per-character autostart preference lets selected characters resume automatically through the existing user-service manager, without opening a dashboard for each one.

## What Changes

- Add `gyro autostart <id> on|off` and expose the current preference in `gyro status`, including JSON output.
- Add a confirmed dashboard autostart toggle and visible On/Off/Unavailable status in full and compact layouts.
- Use persistent systemd user-service enablement; changing autostart does not start or stop a worker. Existing Start/Stop/Recover behavior stays unchanged.
- Remove persistent autostart registration before deleting an inactive character, preventing stale startup units.
- Document automatic startup at login and the separate, optional account-wide lingering setup needed for boot-before-login operation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `local-character-runtime`: Persistent per-character autostart controls and status through the user-service manager.
- `terminal-dashboard`: A separate confirmed autostart action and presentation, independent of runtime activity.
- `managed-character-administration`: Clean up autostart enablement when deleting an inactive character.

## Impact

Touches lifecycle integration in `src/lifecycle.rs`, CLI dispatch/status/deletion in `src/cli.rs`, ownership-protected deletion in `src/runtime.rs`, dashboard provider/input/rendering in `src/dashboard.rs`, and their existing tests. Updates `README.md` and `docs/linux-release-install.md`. The packaged service already declares `WantedBy=default.target`; no additional daemon, dependency, or character-store migration is needed.

## Non-goals

Automatically configuring lingering, installing services, elevating privileges, remembering which workers were running at shutdown, automatic crash recovery, bulk toggles, and changing simulation or offline-time progression.
