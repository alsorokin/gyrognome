## Why

Gyrognome can now demonstrate browser-equivalent report behavior and safe
online enrollment observations, but imported browser characters cannot submit
their already-established progress through an intentional local action. Users
need a narrowly scoped, opt-in reporting path before native enrollment exists.

## What Changes

- Add an explicit command for submitting a single browser-equivalent
  leaderboard report for a selected, imported managed character.
- Require an interactive per-submission confirmation and refuse reporting for
  offline-originated characters, active local runtimes, invalid credentials, or
  incomplete enrollment-conformance evidence.
- Derive report payloads from the current persisted canonical state, send them
  only to the official endpoint, and record a credential-free local submission
  outcome without exposing browser save data or passkeys.
- Keep routine worker advancement, dashboard refreshes, registration, and
  character administration transport-free. Native online creation remains out
  of scope.

## Capabilities

### New Capabilities

- `opt-in-leaderboard-reporting`: Provides a deliberate, credential-safe,
  one-shot reporting workflow for browser-imported managed characters.

### Modified Capabilities

- `local-character-runtime`: Allows the explicit reporting workflow as the
  sole exception to its no-HTTP runtime boundary while preserving transport-free
  background execution.

## Impact

- Affected systems: CLI commands, managed-character persistence, report
  construction, HTTP transport, explicit confirmations, and credential-safe
  diagnostics.
- Adds a minimal HTTP client dependency only for the explicit reporting path.
- Does not add native character enrollment, automatic report scheduling, or
  expose credentials through CLI output, evidence, or logs.
