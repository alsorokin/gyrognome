## Why

Manual bragging currently fails with an `already running` error whenever the
selected character is owned by its local worker, even though other explicit
online actions already operate safely while simulation continues. This makes
the dashboard's immediate Brag action unusable in the character's normal
running state.

## What Changes

- Allow an eligible manual-brag report while the managed character's local
  runtime is active or inactive.
- Serialize manual bragging with other online actions without stopping,
  restarting, or interrupting the worker.
- Use the latest persisted character and profile snapshot for the one-shot
  report while preserving the existing credential-safety, conformance, and
  official-endpoint gates.
- Apply the behavior consistently to the dashboard Brag action and the
  confirmed `gyrognome report` command.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `opt-in-leaderboard-reporting`: Manual brag eligibility and active-runtime
  behavior change from refusal to serialized submission without interrupting
  runtime ownership.

## Impact

- Affects the shared reporting target acquisition in `src/reporting.rs` and
  `src/runtime.rs`.
- Changes user-visible behavior for dashboard bragging and the CLI `report`
  command when a worker owns the character.
- Requires reporting, runtime-boundary, dashboard, and CLI regression coverage.
- Introduces no new dependency, protocol format, endpoint, or persisted schema.
