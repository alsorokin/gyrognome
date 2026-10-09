# Linux release: install and upgrade

Published archives and adjacent checksum files are available from the
[GitHub Releases page](https://github.com/alsorokin/gyrognome/releases).
No npm package, crates.io crate, AUR package, `.deb`, or `.rpm` is available;
`package.json` is solely a Node.js conformance-test harness. Candidate archives
produced by the manual GitHub Actions workflow are temporary review artifacts
and expire after seven days. Published and candidate archives include the
project license and both official upstream notices.

Each archive names its Linux architecture (`x86_64` or `aarch64`) and targets
glibc 2.35 or newer. Native Ubuntu 22.04 candidate builds required at most
GLIBC 2.34 on both architectures; compatibility was trialed on glibc 2.35.
Support for
older glibc releases, musl distributions, and non-Linux systems is not claimed.
Check your machine with `uname -m` and `getconf GNU_LIBC_VERSION` before
selecting the matching archive. The build process rejects binaries that
require GLIBC versions newer than 2.35; a working install on your distribution
also requires a compatible Linux userland. ARM installation was trialed under
emulation after a native ARM build and executable smoke check; its
`systemd --user` lifecycle has **not** yet been trialed on a native ARM host.

## Install without Rust or root access

With the archive and its adjacent `.sha256` file in the same directory:

```sh
sha256sum -c gyrognome-1.0.0-x86_64-unknown-linux-gnu.tar.gz.sha256
tar -xzf gyrognome-1.0.0-x86_64-unknown-linux-gnu.tar.gz
cd gyrognome-1.0.0-x86_64-unknown-linux-gnu
mkdir -p "$HOME/.local/bin" "$HOME/.config/systemd/user"
install -m 755 gyro gyrognome "$HOME/.local/bin/"
"$HOME/.local/bin/gyro" --help
```

Use the `aarch64-unknown-linux-gnu` filename on ARM64. Substitute the actual
archive version; verify a digest from a trusted channel before using any
download. Running `gyro` without its absolute path requires `~/.local/bin` to
be in your shell's PATH; this is not required for the user service. Keep the
archive and the previous binaries until you know the new version works.

**Optional user service:** `gyro start/status/stop/recover` require an active
`systemd --user` manager. From inside the extracted directory, use the bundled
unit and set its executable to the absolute installed path:

```sh
case "$HOME" in *' '*|*'%'*|*'|'*|*'\'*) echo "Unsupported home path for this unit recipe" >&2; exit 1;; esac
sed "s|^ExecStart=gyro worker %i$|ExecStart=$HOME/.local/bin/gyro worker %i|" \
  gyrognome@.service > "$HOME/.config/systemd/user/gyrognome@.service"
systemctl --user daemon-reload
```

The unit is not enabled or started by installation. Register a character with
`"$HOME/.local/bin/gyro" register /path/to/save.pqw` and use the returned ID
with `"$HOME/.local/bin/gyro" start <id>`; the service manager then launches
that installed executable, independently of its PATH. Avoid importing a real
save into an untrusted test environment: online-originated saves may contain
bearer credentials. If `systemd --user` is unavailable, run
`"$HOME/.local/bin/gyro" worker <id>` directly in a foreground terminal.
Service lifecycle commands do not work without a user manager.

**Optional per-character autostart:** After installing the service template,
use `gyro autostart <id> on` for each character you want to resume automatically.
Use `gyro autostart <id> off` to disable it and `gyro status <id>` to inspect it.
The dashboard shows the same preference and offers an `a` toggle (`auto` in
key help), confirmed with Enter or cancelled with Escape.
Registration and installation leave autostart off unless the instance was
already enabled externally.

This is persistent systemd enablement, separate from current activity: On does
not start an inactive worker, Off does not stop an active worker, and
Start/Stop/Recover do not change it. An enabled character starts when your user
manager next starts, normally at login, even if you previously stopped it.
Unit-file preference changes do not require a running user manager.

For startup after reboot **before login**, and continued operation after logout,
separately run `loginctl enable-linger "$USER"` if permitted by the host. This
account-wide change may require administrator authorization; Gyrognome never
makes it automatically. Without lingering, On is not a promise of pre-login
startup. Keep the absolute executable path configured above valid.
Workers resume saved state with existing rested-time and reporting gates,
not instant offline progress.

## Upgrade, backup, and rollback

Stop active managed-character services with `gyro stop <id>` (or stop a direct
worker) and record which IDs were running. Back up the private XDG store:
`${XDG_DATA_HOME:-$HOME/.local/share}/gyrognome/` contains credentials and
must remain private. For example, after stopping all workers:

```sh
data_dir="${XDG_DATA_HOME:-$HOME/.local/share}/gyrognome"
backup_dir="$HOME/gyrognome-backup-before-upgrade"
test -d "$data_dir" && test ! -e "$backup_dir" || { echo "No store or backup already exists" >&2; exit 1; }
umask 077
cp -a "$data_dir" "$backup_dir"
```

Autostart enablement survives binary upgrades and rollback. If rolling back to
a version without the autostart command, disable an instance with
`systemctl --user disable gyrognome@<id>.service` without `--now`; use Stop
separately if needed. Confirmed character deletion removes its startup
registration before deleting data. If startup cleanup fails, the character
remains registered.

Keep this backup outside the repository and inaccessible to other users.
Replace the two
executables in `~/.local/bin` using the steps above; retain the previous
binaries and unit. Reload the user manager if the unit was replaced, then
restart only the workers that were active. Never replace or delete the XDG
data store as part of a binary install.

If the new binary fails, stop workers and restore the old binaries/unit. If
opening the store migrated its schema, an older binary might not understand
it: restore the `characters.sqlite3.pre-v<schema>.backup` snapshot to a
private `0700` data directory as `characters.sqlite3` with mode `0600`, as
described in the repository README. That restore loses progression recorded
after the snapshot; keep a separate backup of the newer store.
