# Tasks

## 1. Shared autostart controls and CLI

- [x] 1.1 Extend `src/lifecycle.rs` and its injectable runner with persistent enable/disable and unit-file status parsing; verify fake-runner tests cover enabled, disabled, runtime-only, unsupported/missing, command failures, repeated settings, and no Start/Stop calls.
- [x] 1.2 Add `gyro autostart <id> on|off` and additive text/JSON autostart status in `src/cli.rs`; verify parser and CLI integration tests cover both settings, invalid/unknown identifiers, actionable failures, and independent activity/ownership fields.

## 2. Dashboard and deletion integration

- [x] 2.1 Wire autostart into dashboard providers/snapshots, the service-status refresh schedule, `a` confirmation/Enter/Escape handling, and full/compact status and key help; verify existing dashboard tests plus new cases for cancellation, unchanged worker activity, external refresh, unavailable state, message retention, and narrow-layout rendering.
- [x] 2.2 Coordinate startup cleanup with ownership-protected deletion in `src/runtime.rs` and `src/cli.rs`; verify cancellation and active/owned protection leave enablement untouched, enabled deletion removes registration, cleanup failure preserves data, post-cleanup removal failure reports disabled autostart, and absent optional service integration still permits deletion.

## 3. Documentation and integration check

- [x] 3.1 Update `README.md` and `docs/linux-release-install.md` with CLI/dashboard usage, independent Start/Stop semantics, opt-in defaults, rollback, installed executable path, and login versus explicit optional lingering; verify examples match CLI help and make no unconditional pre-login startup promise.
- [x] 3.2 Run the focused lifecycle/dashboard and CLI/runtime-boundary checks, formatting and compile checks; verify persistent enable/disable target links with isolated synthetic data/unit configuration and unchanged runtime activity, without rebooting the shared host, altering lingering, or using real saves.
