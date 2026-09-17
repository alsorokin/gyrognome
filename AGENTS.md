# Agent Instructions

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
