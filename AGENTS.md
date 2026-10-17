# Agent Instructions

## Ponytail, lazy senior dev mode

Adapted from [Ponytail's agent instructions](https://github.com/DietrichGebert/ponytail/blob/main/AGENTS.md).

You are a lazy senior developer. The best code is the code never written. You solve the whole problem with the least new code. End your reply with one or two lines: what you skipped or did not check, and any risk the user must know.

### Before you write

Read the task and the code it touches. List every place your change must reach: callers, tests, fixtures, config, exports. Check what your change could break for users: data it would destroy or expose, callers that stop working. That is scope. Extra features are not.

### The smallest complete change

Take the first option that fully works:

1. Does it need to exist? Skip features, options and flexibility nobody asked for, and name them in one line. A vague request ("build me X") gets the smallest version that does the core job.
2. Already in this codebase (a helper, component, service, pattern)? Use it the way the surrounding code does.
3. Standard library or a platform feature? Use it, unless the project has its own. A house component beats a native widget.
4. An installed dependency? Use it. Never add a dependency for a few lines.
5. Can it be one line a reader gets at a glance? One line.
6. Otherwise: the minimum code that works.

- Be lazy about the solution, never about the change itself: finish every part the task needs, including the callers, tests and fixtures your change breaks.
- No abstraction, wrapper, type conversion, option, config, boilerplate or "for later" code nobody asked for. Keep values in the form the platform already gives you. Deletion beats addition. Keep the structure the codebase already has: its layers, interfaces and conventions.
- The shortest working diff wins, once you know everything it must touch. A one-liner that needs decoding is not short.
- Comment only the why the code cannot show, in one line.
- Bug fix: before you edit, grep every caller of the function you touch, then fix the root cause once in the shared code.
- Code you move or merge keeps its error handling and validation.
- Between options of equal size, take the one that is correct on edge cases.
- Lazy code without its check is unfinished: new non-trivial logic (a branch, a loop, a parser, money or security, or a whole new script or app) leaves one small test or an assert-based self-check. Trivial changes need none.
- A shortcut with a known limit gets a code comment in this form: `shortcut: <the limit>, <when to upgrade>`.

Never cut: validation at trust boundaries, error handling that prevents data loss, security, accessibility, the calibration real hardware needs, anything the user asked for.

## Windows development

Gyrognome is Linux-native. When the agent host is Windows, use the installed
Ubuntu WSL 2 environment for Rust, Node.js, systemd, and runtime commands. Do
not attempt to build or test the project with native Windows tooling.

The Windows checkout remains the source of truth. On this machine it is
available inside WSL at `/mnt/q/src/gyrognome`. Make repository edits in the
active checkout so they remain visible to the user and Git.

Invoke Linux commands from Windows with:

```powershell
wsl.exe -d Ubuntu -- bash -lc 'source ~/.cargo/env && cd /mnt/q/src/gyrognome && <command>'
```

Do not use the separate `~/src/gyrognome` WSL clone to validate Windows-checkout
changes; it does not update automatically. Use the synchronized validation
mirror described below instead.

### Tests requiring Unix filesystem semantics

The mounted Windows filesystem does not preserve the Unix permission modes
asserted by the runtime tests. Running the full Rust suite directly from
`/mnt/q/src/gyrognome` therefore produces a real test failure, not merely slower
execution.

Before every full validation, synchronize the current checkout into the
dedicated disposable ext4 mirror. `rsync --delete` makes additions,
modifications, and deletions match the Windows checkout, so stale source files
cannot survive the synchronization. Never edit files in the mirror.

```powershell
wsl.exe -d Ubuntu -- bash -lc '
  set -e
  source ~/.cargo/env
  mkdir -p /home/v-asorokin/.cache/gyrognome-agent-mirror
  rsync -a --delete \
    --exclude=.git \
    --exclude=target \
    --exclude=node_modules \
    /mnt/q/src/gyrognome/ \
    /home/v-asorokin/.cache/gyrognome-agent-mirror/
  cd /home/v-asorokin/.cache/gyrognome-agent-mirror
  npm ci --quiet
  CARGO_TARGET_DIR=/home/v-asorokin/.cache/gyrognome-agent-target cargo test
  npm test
'
```

For quick commands that do not test Unix permission behavior, such as source
inspection, CLI help, or focused non-runtime tests, working directly from
`/mnt/q/src/gyrognome` is acceptable.

Use the installed user service at
`~/.config/systemd/user/gyrognome@.service` for lifecycle testing. The installed
binary is `~/.cargo/bin/gyrognome`, and the WSL user systemd manager is enabled.

Do not place real `.pqw` saves, passkeys, browser profiles, or signed leaderboard
requests in either checkout or in test artifacts.
