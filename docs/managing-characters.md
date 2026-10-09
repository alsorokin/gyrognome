# Managing characters

The [README](../README.md) covers everyday commands. This guide explains
service setup, recovery, online-action outcomes, and private storage.

## Background services

`gyro start`, `stop`, `status`, and `recover` use your systemd user manager;
they do not require a system-wide service or root access.
Install the service template and configure the binary's absolute path using the
[archive installation guide](linux-release-install.md#install-without-rust-or-root-access).
The user manager must be available for lifecycle commands to work.

```sh
gyro start <character-id>
gyro status <character-id>
gyro stop <character-id>
gyro recover <character-id>
```

`status` shows service activity, whether a worker owns the character, and the
autostart preference. `recover` clears a failed service state and starts a new
worker from the last saved progress. Fix the underlying error before recovering
or the service may fail again.

Only one worker can run a character at a time. A second worker fails with
"already running"; the lock is released when the first exits or crashes.
The `gyrognome` executable remains available for older service configurations;
use `gyro` for interactive commands.

### Foreground operation

Without systemd, run:

```sh
gyro worker <character-id>
```

Ctrl-C stops it cleanly. Service controls cannot manage this foreground worker,
and it must be stopped manually before export or deletion.
`--interval-ms` adjusts the worker's update interval (default: 1000 ms), not
the game's progression multiplier.

## Automatic startup

Autostart is opt-in per character:

```sh
gyro autostart <character-id> on
gyro autostart <character-id> off
gyro status <character-id>
```

An enabled character starts when your user manager next starts, normally at
login. Enabling does not start it now; disabling does not stop it.
Start, Stop, Recover, and export do not change this preference. A manually
stopped character with autostart On will resume at the next login.

To run enabled characters after boot before login and keep them running after
logout, separately enable account-wide lingering:

```sh
loginctl enable-linger "$USER"
```

Your host may require administrator authorization. Gyrognome does not enable
lingering or elevate privileges automatically. The installed service template
and its executable path must remain valid.

Autostart changes can be made without an active user manager. The dashboard
and CLI reflect persistent changes made through systemd as well; temporary
`--runtime` enablement does not count as autostart.

Startup resumes saved progress rather than granting instant offline rewards.
Desktop characters retain their online-play restrictions, including the warning
about permanent local-only advancement.

Deletion disables a character's autostart before removing its data. If that
cleanup fails, the character remains registered. Autostart also survives binary
rollback; an older version without the CLI toggle can disable it with
`systemctl --user disable gyrognome@<character-id>.service`.
Stop it separately if needed.

## Rested progression

Characters earn rested time while stopped or while the computer sleeps or
hibernates, capped at 12 hours. Each banked second buys one real second of 2x
progression. Unused rest survives stopping; new registrations start empty.
Motto and guild edits do not reset it.

The dashboard shows available rest and the active progression multiplier.
A stopped character can have rest without being actively boosted.
Rest stays in the managed database and is not included in official save exports.
Leaderboard acceptance of accelerated timelines is unverified.

Slow reporting or scheduler delays do not earn rest or become catch-up progress.
Stopped-time accounting uses the local clock, so clock changes can affect the
bank. After a crash, downtime is estimated from the last saved checkpoint.

## Online actions

Creating an online character requires the interactive `gyro new-guy` wizard.
Passing `--name`, `--race`, and `--class` together is always offline-only.
A duplicate online name returns you to the editable draft; other enrollment
failures are reported without automatically retrying or registering a local
replacement.

Manual `gyro report` requires an eligible online character and no active worker.
It asks you to type `yes`, sends one report, and opens the realm's public
leaderboard page after the attempt, even if delivery fails. Declining sends
nothing and opens no page. Dashboard Brag submits immediately without a
confirmation dialog. Automatic level and act reports never open a browser.

Motto and guild changes can be submitted while a worker is active. Browser
profile text accepts printable Unicode; desktop text must be ASCII. Empty motto
text or `--clear` clears the motto; an empty guild designation leaves the guild.

A motto is saved before delivery and remains saved even if the request fails.
Later reports use that value. A guild is saved only when acceptance is confirmed;
rejected or uncertain results preserve the previous membership. Pemptus guild
actions check the public leaderboard, which can lag behind the request.

Requests are attempted once, without automatic retries. A failed or uncertain
attempt may already have reached the server; delivery alone does not establish
acceptance or normal leaderboard classification.
Inspecting, registering, refreshing the dashboard, and managing services do not
independently submit reports.

For desktop restrictions and permanent local-only play, see
[classic desktop compatibility](classic-desktop-compatibility.md#online-play).

## Storage, backup, and rollback

The private data directory is `$XDG_DATA_HOME/gyrognome/`, or
`~/.local/share/gyrognome/` when `XDG_DATA_HOME` is unset. It contains
`characters.sqlite3` and worker/action lock files. The database holds
credentials and retained browser save data; desktop saves are stored as managed
state and credentials rather than the original binary file.

Stop all workers before backing up the directory. Keep backups private and
outside repositories. The
[installation guide](linux-release-install.md#upgrade-backup-and-rollback)
includes a backup recipe and binary upgrade steps.

Before migrating the database, Gyrognome creates a private
`characters.sqlite3.pre-v<schema>.backup` snapshot. An older binary may not
understand the migrated database. To roll back, stop all workers, preserve the
newer data directory elsewhere, and restore the snapshot as
`characters.sqlite3` in a private directory with mode `0700` and file mode
`0600`. Restore the older binary and service configuration before starting
workers. Progress recorded after the snapshot will not be present.
Gyrognome does not automatically downgrade the database.

## Inspection and dashboard options

`gyro inspect <save>` reads a source file without modifying or registering it.
`gyro managed-inspect <character-id>` reads saved managed progress.
Both support `--json` and omit credentials. Managed JSON includes rested-time
details; desktop inspection also shows compatibility and online availability.

The dashboard refreshes every second by default; adjust this with
`--refresh-ms` (100 to 60000). Its animated task bar is a display prediction,
not extra progression. Other rewards and statistics update from saved state.
Quitting the dashboard does not stop a worker.
