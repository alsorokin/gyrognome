# Design

## Context

See [proposal.md](proposal.md) for motivation. The dashboard currently performs
periodic combined refreshes, additional persisted-state reads near predicted
task completion, and immediate refreshes after successful actions. It also maps
`r` to the same combined refresh and exposes recovery independently on `c`.
The lifecycle layer distinguishes inactive and failed service states, and its
recovery operation runs `systemctl --user reset-failed` before the normal
validated start path.

## Goals / Non-Goals

**Goals:**

- Make the dashboard controls reflect actions that are useful for the currently
  displayed service state.
- Preserve recovery from systemd failed-state and start-rate-limit conditions.
- Keep all existing automatic and post-action refresh behavior.
- Publish the completed dashboard behavior as project version `1.1.0` with
  consistent Rust and Node package metadata.

**Non-Goals:**

- Removing or changing the `gyro recover` command.
- Changing systemd unit behavior, lifecycle error handling, or simulation
  persistence.
- Reassigning the freed `r` or `c` keys.

## Decisions

### Remove the manual refresh command rather than hiding only its help text

The dashboard command enum, key mapping, command application branch, footer
help, and dedicated tests will all stop treating `r` as an action. This avoids
an undiscoverable duplicate command and ensures the visible controls match the
actual input behavior.

Keeping the key as an undocumented fallback was rejected because automatic
refresh already bounds staleness and hidden controls make the interaction model
harder to test and explain.

### Select recovery from the observed failed service state

The existing lifecycle toggle will map:

| Displayed service state | Confirmed action |
|-------------------------|------------------|
| Active                  | Stop             |
| Inactive                | Start            |
| Failed                  | Recover           |
| Unavailable             | Start             |

This keeps the operationally distinct recovery implementation while presenting
one user concept: change the runtime toward the appropriate healthy state.
Mapping failed to ordinary start was rejected because `reset-failed` can be
required after repeated failures or systemd start-rate limiting.

### Keep recovery in the lifecycle abstraction and CLI

`LifecycleAction::Recover`, the dashboard provider operation, and
`Lifecycle::recover` remain because the contextual failed-state action still
uses them. The explicit CLI command remains useful for troubleshooting and
automation when the dashboard is unavailable.

### Update all authoritative package version metadata together

Set the package version to `1.1.0` in `Cargo.toml` and `package.json`, then use
the ecosystem package managers to synchronize the Gyrognome package entries in
`Cargo.lock` and `package-lock.json`. Treat mismatched manifest and lockfile
versions as an incomplete change.

Updating only the Rust version was rejected because the repository's Node
package metadata participates in its validation and release tooling even
though the shipped application is Rust-native.

## Risks / Trade-offs

- **A user accustomed to `r` may initially expect it to work** -> Remove it
  from documentation and help consistently; the next bounded automatic refresh
  supplies the expected update.
- **Service status may be temporarily stale when the lifecycle key is pressed**
  -> Preserve the existing bounded service-status polling and confirmation so
  the selected action is visible before execution; lifecycle errors remain
  explicit if the state changed.
- **The footer label `start/stop` would under-describe failed-state behavior**
  -> Use a neutral lifecycle label or a state-specific label that includes
  recovery when the displayed service is failed.
- **A partial version update could leave release metadata inconsistent** ->
  Regenerate both lockfiles through Cargo and npm and verify all root package
  entries report `1.1.0`.

## Migration Plan

Update the dashboard interaction, tests, README, terminal-dashboard spec, and
package metadata together. No data or service migration is required. Rollback
consists of restoring the two key mappings, the previous failed-state action
selection, and version `1.0.0` consistently across the manifests and lockfiles.
