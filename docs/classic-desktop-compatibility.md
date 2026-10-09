# Classic Desktop Save Compatibility

Gyrognome can continue supported saves from the original Windows Progress Quest
client and export your progress back to a save that Progress Quest 6.4.4 can
open. Desktop characters keep the classic game's rules; they are not converted
to browser characters.

## Import a character

Supported saves include the original 6.2 and 6.4.4 `.pq` formats and `.bak`
backups in the same format. Unsupported or damaged saves are rejected.
Desktop support currently requires ASCII text, including names, credentials,
mottos, and guilds.

Keep a backup of your save and **stop the classic client before continuing the
character in Gyrognome**. Do not run both clients for the same character at once.

```sh
gyro register /path/to/character.pq
```

You can use a `.bak` file instead. Registration makes a managed copy and leaves
the source file unchanged; it does not start the character or send a leaderboard
report.

Use the character ID printed by registration in the commands below.
`gyro list` shows your registered characters and their IDs.

```sh
gyro dashboard <character-id>
gyro start <character-id>
gyro stop <character-id>
```

The dashboard shows progress and available online actions. Starting and stopping
the character's background service requires the
[local runtime setup](managing-characters.md#background-services).

## What to expect

Both supported save versions continue using the classic 6.4.4 rules. Gyrognome
applies the classic client's spelling corrections and supported older-save
adjustments without changing your original file.

The next random events may differ from those in the classic client: desktop
saves do not include its random-number state. Task and elapsed-time counters
start at import because the save does not contain reliable lifetime totals.

At normal speed, progression runs at roughly 91% of wall-clock time, matching
the classic Windows timer. A full task bar may briefly wait before completing.
Stopped time does not grant instant progress, but it earns
[rested time](managing-characters.md#rested-progression) for faster progression
after restarting. A crash can lose partial progress on the current task;
stopping normally saves it.

## Online play

**Spoltog and Pemptus are supported for eligible imports**, including automatic
level and act reports, manual bragging, motto changes, and guild changes.
Spoltog requires the save's account and password; Pemptus uses its saved passkey
without an account or password.

## Export and return to the classic client

Export writes your current managed progress as a new `.pq` save for Progress
Quest 6.4.4, including your motto, guild, and saved login credentials. It is not
a byte-for-byte copy of the original file and does not convert the character
to a browser `.pqw` save.

```sh
gyro export <character-id> -o /path/to/character.pq
```

Without `-o`, the file is written in the current directory using the character's
save name. An existing file requires confirmation; `--force` overwrites it
without asking. Keep your original backup rather than overwriting it.

If the character's background service is running, export temporarily stops it
and restarts it afterwards. A manually launched worker must be stopped first.
**To switch clients, stop Gyrognome before exporting and keep it stopped while
the classic client is running:**

```sh
gyro stop <character-id>
gyro export <character-id> -o /path/to/character.pq
```

Then open the exported file in Progress Quest 6.4.4. To switch back later, stop
the classic client and register its latest save, not the old managed copy.

Exports do not carry Gyrognome's rested-time bank, since-import counters, or
random-number continuation. They also do not guarantee leaderboard acceptance
or remove the limitations on local-only play.

**Treat save files as private:** they contain credentials. Exports are written
with owner-only permissions, but copying them elsewhere may change those
permissions. Do not publish them or commit them to a repository.
