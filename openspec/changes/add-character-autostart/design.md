# Design

## Context

See `proposal.md` for motivation and the three capability deltas for behavior.
`Lifecycle` already wraps an injectable `ServiceRunner` and validates character
identity before service operations. Its status currently contains activity and
ownership only. The packaged `gyrognome@.service` declares
`WantedBy=default.target`, so persistent instance enablement supplies startup
without another daemon.

The dashboard has separate state and service refresh paths, injectable providers,
confirmed lifecycle actions, and full/compact render tests. `Store::remove`
holds an advisory ownership lock around its atomic transaction; the CLI currently
calls it directly after confirmation.

## Goals / Non-Goals

**Goals:** One systemd-backed preference shared by CLI and dashboard; independent
activity and startup status; deletion without dangling enabled instances.

**Non-Goals:** No database preference mirroring, simulation changes, automatic
account-wide boot configuration, service installation, or restart policy changes.

## Decisions

### 1. Systemd unit-file enablement is the source of truth

Extend the existing runner with enable, disable, and is-enabled operations for
the validated `gyrognome@<id>.service` instance. Enable/disable must not use
`--now` or `--runtime`. Query unit-file state on demand, including after setting
the preference, rather than storing a separate database flag that could drift
from external `systemctl` changes.

Parse recognized unit-file output, not just the exit code: persistent `enabled`
maps to On, `disabled` to Off, and runtime-only enabled state is not persistent
autostart. Unsupported states such as masked/static/indirect and missing units
are Unavailable with a diagnostic. Setting On/Off is idempotent; report
configuration failures explicitly. No automatic unmasking or other repair.

An additive `autostart` field in runtime status uses
`enabled|disabled|unavailable`, with an optional diagnostic. Preserve separately
read activity and ownership if only the enablement query fails, and vice versa
in dashboard collection. A failed query must never become Off. Keep existing
CLI error behavior for inability to inspect runtime activity.

### 2. Separate current activity from future startup

Use explicit `gyro autostart <id> on|off`; `gyro status` is the inspection command.
Do not add confirmation to this explicit CLI setter. The dashboard uses `a`
and reuses its confirmation flow with enable/disable actions and explanatory
labels. Refresh enablement on the existing service-status schedule, not every
predicted task-bar redraw.

Both dashboard layouts show autostart separately from activity. Add the shortcut
to bold-key help and use the existing width-aware wrapping, rather than
increasing pane heights unconditionally. Unavailable preference blocks the
dashboard toggle and displays its diagnostic; do not infer a setting from
worker activity. An explicit CLI setter can still be attempted and report its
own error. Action messages follow the existing five-second retention.

Start, Stop, Recover, and export's temporary stop/restart remain untouched.
Enabling a stopped character leaves it stopped until the next user-manager
startup or explicit Start. A manually stopped enabled character will run again
at the next user-manager startup; this is not last-running-state persistence.

### 3. Keep boot-before-login setup explicit

Documentation explains that ordinary user-service startup occurs at login.
For operation before login and after logout, users may separately run
`loginctl enable-linger "$USER"` subject to host authorization. Gyrognome never
does this automatically, because lingering affects the entire user account.
Document the absolute executable path prerequisite and existing service
installation recipe. No diagnostic claim that On guarantees pre-login startup.

### 4. Clean up enablement under the existing deletion ownership guard

After deletion confirmation, acquire and hold the character's existing advisory
lock before checking/cleaning startup registration, and hold it through data
removal. Extend the existing removal path narrowly to allow this coordinated
cleanup; do not use a racy pre-check followed by a separate lock acquisition.
An active systemd instance must also be rejected before cleanup. Do not invoke
cleanup at all for cancellation or an owned/active character.

Use systemd unit-file operations, which can operate without a running user bus,
for cleanup; a stopped user manager alone must not make direct-worker deletion
depend on an active session. A proven absent template and absent startup
registration is a no-op. Preserve deletion on installations without optional
service integration; absence of integration must be established, not inferred
from a generic command failure.

Abort data removal on genuine cleanup failure. If cleanup succeeds but atomic
data removal fails, retain the record and explicitly report that autostart has
been disabled. Do not attempt automatic re-enablement: two independent systems
cannot share an atomic transaction, and preserving character data is the more
important guarantee.

## Risks / Trade-offs

- On does not imply immediate activity or boot-before-login operation -> Label
  autostart independently and document login/lingering semantics.
- Systemd has more unit-file states than a boolean -> Parse recognized states
  and preserve diagnostics for unsupported or unavailable configuration.
- Deletion spans filesystem configuration and SQLite -> Hold ownership through
  both steps; clean startup first; explicitly report partial configuration change.
- Automatic startup bypasses interactive startup warnings -> Enabling copy and
  documentation must explain that later workers use existing reporting gates
  and local-only provenance rules; do not grant new eligibility or suppress
  existing notices.

## Migration Plan

No schema migration or automatic enablement. Existing external enablement is
recognized. Users opt in per character after installing the update. Rollback to
an older binary leaves systemd enablement in place; it can be removed with
`systemctl --user disable gyrognome@<id>.service`, independently of the binary.

## Validation

Use fake runners/providers for enablement parsing, idempotency, configuration
failures, action confirmation/cancellation, refresh, and both render layouts.
Extend CLI integration tests for parsing, text/JSON status, and deletion failure
and ownership protection. With isolated synthetic character data and an
isolated user-unit configuration, verify that persistent enablement creates the
expected startup link and disable removes it without changing worker activity.
Do not reboot the shared host, restart its whole user manager, change lingering,
or use real saves for validation. Existing worker restart coverage verifies
persisted-state continuation; inspect the enabled instance's target wiring for
automatic launch.
