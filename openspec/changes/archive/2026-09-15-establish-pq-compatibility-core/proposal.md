## Why

A Linux-native Progress Quest client needs a trustworthy compatibility foundation before it can safely advance or report an online character. The official web client exposes a state format and signed leaderboard protocol, but native behavior must be proven deterministic and must not risk a player's existing online character.

## What Changes

- Establish a Rust compatibility core for canonical Progress Quest character state and browser `.pqw` save interchange.
- Provide deterministic, Alea-compatible random-number generation and state restoration needed to reproduce browser simulation behavior.
- Define browser-compatible URL encoding and LFSR validator construction for leaderboard requests without submitting live reports.
- Introduce sanitized, credential-free reference fixtures and conformance tests derived from the official browser client.
- Provide a read-only character inspection interface for validating imported saves.

## Capabilities

### New Capabilities

- `pq-compatibility-core`: Imports and exports browser-compatible saves, models canonical character state, restores deterministic random state, and constructs testable signed leaderboard requests without network submission.

### Modified Capabilities

- None.

## Impact

- Adds the initial Rust project, core state, serialization, compatibility primitives, fixtures, and tests.
- Establishes the module boundary later used by persistence, a terminal UI, a systemd user service, and a network-enabled leaderboard client.
- Uses the existing repository-scoped Playwright MCP only to observe the official browser client and produce redacted reference data.
- Does not transmit credentials, create online characters, or send leaderboard reports.
