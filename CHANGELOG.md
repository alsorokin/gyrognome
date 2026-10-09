# Changelog

All notable changes to Gyrognome are documented in this file.

## [Unreleased]

### Added

- `export <id> [-o PATH] [--force]` writes a managed character back to a save:
  `.pq` for desktop characters (opens in pq.exe 6.4.4) and `.pqw` for browser
  characters, with stored credentials, an overwrite prompt, and atomic 0600
  writes. A running service is stopped for the export and restarted.
- Automatic rested progression for all managed browser and desktop characters:
  stopped or sleeping time builds a capped 12-hour bank, spent one real second
  at a time for 2x progression. Includes durable fractional timing, sleep-aware
  runtime pacing, and rested status in inspection, both dashboard layouts,
  and character selection.
  Existing records begin with an empty bank on migration. Eligible online
  delivery retains existing gates; rested-timeline leaderboard acceptance and
  classification remain unverified.

- Open the selected character's official realm-specific public leaderboard
  page in the system browser after a confirmed manual brag, without changing
  the report outcome if browser opening fails.
- Pemptus desktop online support for unadapted, spelling-only, and exact indexed
  quest-placeholder-only imports, including existing unadvanced managed characters:
  automatic level/act reporting, manual brag, motto set/clear, and guild
  join/change/leave. Reporting reuses pinned observations through explicit
  supported-path equivalence; guild actions use one native request and bounded
  public confirmation without mutation retries. Unsupported combinations and
  permanent local-only histories remain excluded.
- Independent classic operation eligibility and dashboard presentation of partial
  availability, plus separately authorized native Pemptus conformance stages with
  preserved source provenance and credential-free operation evidence.
- Development-only, feature-gated Pemptus manual-report probe with passkey-only
  HTTPS delivery, explicit disposable-character handoff, no mutation retries,
  and bounded credential-safe public observations. The diagnostic distinguishes
  delivery from independently established acceptance and does not enable
  production Pemptus reporting.
- Keyboard and mouse-wheel scrolling for overflowing full-layout dashboard
  panes and compact Character content, with independent clamped offsets and
  visible keyboard focus.

### Changed

- Simplified dashboard Details by omitting technical ID and compatibility rows
  and the long rested-timeline leaderboard notice (retained in managed inspection).
- Matched desktop runtime pacing to the original callback cadence, with
  completion-, report-, provenance-, and stop-boundary persistence and
  dashboard prediction at the same rate.
- Refined the compact dashboard with current-quest-only display, a dedicated
  task progress bar, and clearer spacing between Character sections.
- Reorganized the full dashboard: the Journal title shows the current plot,
  Status stays visible beside Keys, and F1-F6 toggle Activity, Progress,
  Equipment, Details, Adventure, and Journal. Keys now adapt their height to
  wrapped content while preserving the compact dashboard layout.

## [1.0.0] - 2026-09-30

The first feature-complete release of Gyrognome, a Linux-native Progress Quest
client with browser-compatible simulation, local character management, and
credential-safe opt-in online actions.

### Added

- Browser `.pqw` save import, validation, safe inspection, and compatible
  unmodified export support.
- Browser-compatible Alea random-state continuation, URL encoding and
  normalization, and LFSR request construction.
- Pure deterministic simulation using bundled, versioned Progress Quest rules.
- XDG-scoped SQLite persistence for managed characters, including atomic state
  updates and schema migration support.
- Per-character runtime ownership and `systemd --user` lifecycle controls.
- Offline advancement with monotonic timing, bounded worker intervals, and no
  stopped-time catch-up.
- Credential-safe terminal dashboard with live persisted-state refresh,
  collapsible panes, progress prediction, lifecycle controls, and compact
  terminal layouts.
- Safe character administration, including confirmed deletion of inactive
  managed characters.
- Explicit manual-brag reporting for eligible imported online characters.
- One-shot automatic reports for persisted online level-up and act-completion
  events, with no queues or automatic retries.
- Motto and guild profile persistence, CLI actions, and dashboard editors.
- Motto clearing and guild leaving through explicit empty-value actions.
- Foreground Offline and Online `new-guy` enrollment modes with duplicate-name
  handling and credential-safe failure behavior.
- Browser-derived leaderboard and enrollment conformance fixtures and
  disposable-character Playwright validation workflows.

### Security and privacy

- Passkeys, signed request URLs, raw browser saves, browser profiles, raw
  endpoint responses, and unrecognized source fields remain excluded from
  normal inspection, diagnostics, and dashboard output.
- Online actions are restricted to the official endpoint and validated
  credential-free conformance evidence.
- Guild responses are bounded, normalized, fingerprinted, and exposed only as
  safe accepted, rejected, or indeterminate outcome categories.
