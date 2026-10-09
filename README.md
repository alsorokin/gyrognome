# Gyrognome

<img width="425" height="425" alt="Gyrognome logo" src="https://github.com/user-attachments/assets/e86c6dae-f233-4df9-9d9f-903c0f838417" />

Gyrognome is a Linux-native terminal client for
[Progress Quest](https://progressquest.com/), where a dizzy little gnome keeps
your hero's progress wheel spinning. Create a character or continue an existing
save, watch it adventure in the dashboard, and let it progress in the
background.

<img width="1920" height="1054" alt="Gyrognome terminal dashboard" src="https://github.com/user-attachments/assets/78c91f56-bc8d-48e8-8dd2-3e48f42a645d" />

## Features

- Import browser `.pqw` saves and supported classic desktop `.pq` or `.bak` saves.
- Play offline or use supported online leaderboards: Alpaquil for browser
  characters, and Spoltog and Pemptus for eligible desktop imports.
- Manage characters in a terminal dashboard, including motto and guild changes.
- Run characters in the background, with optional automatic startup at login.
- Earn rested time while stopped or asleep, for up to 12 hours of 2x progression.
- Export current progress back to a browser or classic desktop save.

Desktop characters retain the classic game's rules. See
[classic desktop compatibility](docs/classic-desktop-compatibility.md) for
supported saves and online-play limitations.

## Install

Download a Linux archive from
[GitHub Releases](https://github.com/alsorokin/gyrognome/releases). Builds are
available for x86_64 and ARM64 (`aarch64`), with glibc 2.35 or newer.
There is no npm, crates.io, AUR, `.deb`, or `.rpm` package.

Follow the [installation and upgrade guide](docs/linux-release-install.md) to
verify the download, install `gyro`, and set up the optional systemd user service.
No Rust installation or root access is needed for the archive install.

To build from source instead, see the [developer guide](docs/development.md).

## Get started

### Create or import a character

Open the interactive creator:

```sh
gyro new-guy
```

Choose your character's traits and select **Sold!**. The wizard starts in Offline
mode; select Online to enroll a new browser-profile character with the official
server. Creating classic desktop online characters requires the official client.

Or import an existing save:

```sh
gyro register /path/to/character.pqw
# For a classic desktop save:
gyro register /path/to/character.pq
```

Registration leaves your source file unchanged and does not start playing.
**Stop the original client before running the same character in Gyrognome.**
Keep a backup of the original save.

Creation and registration print a character ID. Use it wherever
`<character-id>` appears below; `gyro list` lists your characters and their IDs.

### Start playing

With the [user service installed](docs/linux-release-install.md#install-without-rust-or-root-access):

```sh
gyro start <character-id>
gyro dashboard
```

Choose a character from the list, or open it directly with
`gyro dashboard <character-id>`. The dashboard shows progress but does not
advance the game itself. Closing it leaves a running character active.

```sh
gyro status <character-id>
gyro stop <character-id>
```

Without systemd, run `gyro worker <character-id>` in a terminal and stop it
with Ctrl-C. You can open the dashboard in another terminal.

### Dashboard controls

| Key | Action |
| --- | --- |
| `q` | Close the dashboard without stopping the character |
| `s` | Start, stop, or recover the character's service |
| `a` | Toggle automatic startup |
| `b` | Brag on the leaderboard immediately, if eligible |
| `m` / `g` | Edit motto / guild |
| `F1` through `F6` | Toggle panes in the full layout |
| `Tab` / `Shift+Tab` | Select a pane to scroll |
| Arrow keys / `PageUp` / `PageDown` | Scroll selected content |

Service and autostart changes require Enter to confirm; Escape cancels.
Profile editors also use Enter to submit and Escape to cancel. Empty text
clears a motto or leaves a guild. The mouse wheel scrolls the pane under the
pointer; narrow terminals use one combined scrollable view.

### Automatic startup and rested time

To resume a character automatically at login:

```sh
gyro autostart <character-id> on
```

Use `off` to disable it. This changes future startup only: it does not start or
stop the character now. Running before login or after logout requires additional
[user-service setup](docs/managing-characters.md#automatic-startup).

Only an active worker advances the game. Stopped time and computer sleep earn
**rested time**, up to a **12-hour bank**. Each banked second gives one real
second of **2x progression** when playing resumes; there are no instant offline
rewards. New characters and imports start with an empty bank.

## Online play

Eligible online characters automatically report level-ups and act completions.
You can also brag manually, change your motto, or join an existing guild:

```sh
gyro report <character-id>
gyro motto <character-id> "Onward!"
gyro motto <character-id> --clear
gyro guild <character-id> "Existing guild designation"
gyro guild <character-id> ""
```

`report` requires confirmation and a stopped worker. It opens the character's
public leaderboard page after the attempt. Motto and guild commands submit
immediately and can be used while playing. Offline-created characters cannot
use these actions; desktop profile text must be ASCII.

Failed requests are not automatically retried, and delivery does not guarantee
leaderboard acceptance. For profile-change behavior and failure
handling, see [online actions](docs/managing-characters.md#online-actions).

## Export and inspect

Export current progress as `.pqw` for browser characters or `.pq` for desktop
characters:

```sh
gyro export <character-id> -o /path/to/character.pqw
```

Omit `-o` to use the default save name in the current directory. Existing files
require confirmation, or `--force` to overwrite without asking.
Export temporarily stops and restarts a running user service; stop a foreground
worker yourself first. **When switching back to the original client, stop
Gyrognome before exporting and keep it stopped.**

Exports include credentials but not the rested-time bank. See the
[desktop export guide](docs/classic-desktop-compatibility.md#export-and-return-to-the-classic-client)
for returning to Progress Quest 6.4.4.

Inspect a save without registering it, or inspect a managed character:

```sh
gyro inspect /path/to/character.pqw
gyro managed-inspect <character-id>
```

Add `--json` for machine-readable output. Inspection does not contact the server
or expose credentials. To remove a character, stop it and run
`gyro delete <character-id>`; deletion requires confirmation.

## Data and privacy

Managed characters are stored in `$XDG_DATA_HOME/gyrognome/`, or
`~/.local/share/gyrognome/` when `XDG_DATA_HOME` is unset.
Back up this directory with all workers stopped to preserve managed progress.

**Saves, exports, and the managed database contain credentials.** Keep them
private and out of Git repositories. Exported saves are written with owner-only
permissions; copying them elsewhere may change those permissions.

See [character management](docs/managing-characters.md) for recovery, backups,
and advanced usage. Run `gyro --help` or `gyro <command> --help` for CLI options.

## License

Gyrognome-authored code is MIT licensed. See [LICENSE](LICENSE) and
[third-party notices](THIRD_PARTY_NOTICES.md) for upstream attribution and the
porting rationale.
