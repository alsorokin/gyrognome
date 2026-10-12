# Design

## Context

`open_public_leaderboard` (`src/reporting.rs`) runs
`Command::new("xdg-open").arg(url).status()`, which inherits the TTY and waits
for exit. Under the generic `xdg-open` path (e.g. Hyprland), xdg-open `env`-runs
the `.desktop` `Exec` in the foreground, and the Brave launcher `exec`s the
browser. When no browser is running, that child becomes the browser itself,
so it writes Chromium logs to the TTY and xdg-open does not exit until the
browser does. `submit_manual_brag_with_opener` then `eprintln!`s any opening
failure, which also writes over the TUI when called from the dashboard
(`dashboard.rs` Brag command, called on the UI thread).

## Goals / Non-Goals

**Goals:**
- No browser output in the terminal; no dashboard blocking; still report quick
  launcher failures.

**Non-Goals:**
- Replacing `xdg-open`, detecting the default browser, or confirming the page
  actually rendered.
- Reporting failures that occur after the 3-second window.

## Decisions

- **Detached spawn.** `Command::new("xdg-open")` with `stdin/stdout/stderr` set
  to `Stdio::null()` and `process_group(0)` (`std::os::unix::process::CommandExt`),
  then `spawn()`. Own process group keeps terminal-generated SIGINT/SIGTSTP
  away from the browser. Alternative `setsid` (full session detach, also
  avoids SIGHUP on terminal close) needs `pre_exec`/unsafe or a `setsid`
  binary; process group is enough for the reported problem, so prefer the
  simpler safe API.
- **Bounded wait in a background thread.** Spawn a thread that polls
  `child.try_wait()` (e.g. every 50 ms) for up to 3 s and sends the result over
  an `mpsc` channel: `Ok(())` on exit 0 or on timeout, `Err` on spawn error or
  nonzero exit. After a timeout the thread keeps calling `child.wait()` so the
  process is reaped and no zombie remains. The opener returns the
  `Receiver<Result<(), PublicLeaderboardError>>` (spawn errors can be sent
  immediately).
- **Callers.**
  - CLI `report`: `recv()` on the receiver (blocks at most ~3 s; usually
    instant when a browser is already running) and keep the existing stderr
    message on failure.
  - Dashboard: `brag` returns the delivery outcome plus the pending receiver;
    `DashboardState` keeps it and `try_recv`s it on each loop iteration/deadline,
    setting the action message "Could not open the public leaderboard page: …"
    on failure. Cap `poll_timeout` while a receiver is pending so the result is
    noticed promptly. Remove the `eprintln!` from the shared brag path.
- **Testability.** Keep the injectable opener seam; tests use a fake opener
  returning a pre-filled receiver. The real launcher's wait/timeout logic is a
  small helper parameterized by the command and timeout so it can be tested
  with `true`, `false`, and `sleep`.

## Risks / Trade-offs

- [Launcher fails after 3 s] -> Treated as success; rare, and acceptable for a
  convenience action.
- [Terminal closed while browser started by gyro is running] -> Process group
  alone does not block SIGHUP from session teardown; acceptable shortcut. Switch
  to `setsid` later if it bites.
- [CLI waits up to 3 s on a fresh browser launch] -> Bounded and only in that
  case; acceptable.
