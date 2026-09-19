# Proposal

## Why

Online managed characters currently advance locally without communicating
browser-equivalent level-up and act-completion events to the leaderboard, and
the dashboard cannot submit a manual brag. Users therefore must leave the
dashboard and use a separate command to submit an update.

## What Changes

- Deliver browser-compatible level-up and act-completion reports after their
  corresponding worker state updates persist.
- Make automatic delivery best effort: attempt each emitted report once, do not
  queue or retry it, and do not roll back progression when delivery fails.
- Add an immediate manual-brag action to the terminal dashboard.
- Remove the local-only, no-automatic-reporting, and no-dashboard-reporting
  constraints while retaining official-endpoint pinning and credential-safe
  output.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `local-character-runtime`: Permit best-effort periodic online report delivery
  from active workers after persisted progression.
- `opt-in-leaderboard-reporting`: Permit automatic level-up and act-completion
  reports and dashboard-initiated manual brag delivery.
- `terminal-dashboard`: Add an immediate manual-brag action and allow its
  foreground report delivery.

## Impact

- Affected systems: deterministic worker trace capture, managed-character
  reporting, terminal dashboard controls, runtime and dashboard tests, and
  roadmap documentation.
- No new external dependency or delivery queue is planned.
