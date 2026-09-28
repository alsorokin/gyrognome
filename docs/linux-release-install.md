# Private Linux candidate: install and upgrade

These instructions describe a **candidate artifact**, not a published download.
No GitHub Release, npm package, crates.io crate, AUR package, `.deb`, or `.rpm`
is available yet. `package.json` is solely a Node.js conformance-test harness.
Do not redistribute a candidate until the maintainer has approved the project
license, upstream-derived material, and the publication review.

The candidate names its Linux architecture (`x86_64` or `aarch64`) and targets
glibc 2.35 or newer. It is built on an Ubuntu 22.04 native runner; support for
older glibc releases, musl distributions, and non-Linux systems is not claimed.
Check your machine with `uname -m` and `getconf GNU_LIBC_VERSION` before
selecting the matching archive. The build process rejects binaries that
require GLIBC versions newer than 2.35; a working install on your distribution
also requires a compatible Linux userland.

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
