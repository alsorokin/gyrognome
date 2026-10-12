# Proposal

## Why

When a manual brag opens the public leaderboard page and no browser is already
running, `xdg-open` runs the browser in the foreground as a child that inherits
Gyrognome's terminal. The browser's log output is written straight over the
dashboard (staircased by raw mode), and the dashboard is left stale or blocked
until the browser exits.

## What Changes

- Launch the system browser fully detached from Gyrognome's terminal: no
  inherited stdin/stdout/stderr and its own process group, so browser output
  never reaches the terminal and terminal signals do not reach the browser.
- Never block the dashboard on the launch. Wait up to 3 seconds in the
  background for an early launcher exit status; a nonzero exit or spawn error
  is reported as a separate, safe opening failure. A launcher still running
  after 3 seconds is treated as a successful open and is reaped in the
  background.
- Show dashboard opening failures as the dashboard action message instead of
  printing to stderr over the TUI. The CLI `report` command keeps printing the
  failure to stderr, waiting at most 3 seconds.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `opt-in-leaderboard-reporting`: the "Open the public leaderboard after manual
  brag" requirement additionally requires the browser launch to be detached
  from the terminal and non-blocking, with a bounded wait for early failures.

## Impact

- `src/reporting.rs`: `open_public_leaderboard` launch and failure reporting.
- `src/dashboard.rs`: Brag action surfaces a pending opening failure as its
  action message.
- `src/cli.rs`: unchanged behavior apart from the bounded wait.
- No new dependencies; uses `std::process` and `std::os::unix` only.
