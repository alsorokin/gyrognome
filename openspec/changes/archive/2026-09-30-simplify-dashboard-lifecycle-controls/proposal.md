# Proposal

## Why

The dashboard already refreshes persisted character and service state
automatically, so its manual refresh control adds noise without a meaningful
user need. Recovery remains useful for rare systemd failed-state and
start-rate-limit conditions, but it should appear as the natural lifecycle
action only when a service has failed rather than as a permanent separate
control.

## What Changes

- Remove the dashboard's manual refresh key and help text.
- Remove the dashboard's dedicated recovery key and help text.
- Make the existing lifecycle key select stop for an active service, start for
  an inactive service, and recover for a failed service.
- Retain automatic dashboard refreshes, post-action refreshes, and the explicit
  `gyro recover <character-id>` troubleshooting command.
- Update dashboard documentation and tests for the simplified controls.
- Bump the project minor version from `1.0.0` to `1.1.0` across Rust and Node
  package metadata.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `terminal-dashboard`: Simplify interactive refresh and lifecycle controls
  while preserving contextual recovery for failed services.

## Impact

The change affects dashboard key mapping, lifecycle-action selection, footer
help, confirmation behavior, dashboard tests, user documentation, and package
version metadata. It does not change persisted state, the systemd service,
lifecycle internals, or the CLI recovery command.
