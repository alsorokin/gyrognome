## Why

Locally registered throwaway characters currently require direct database edits
to remove, and opening the dashboard requires copying a character identifier
for every invocation. Character management should be available through the
credential-safe CLI so routine local administration is safer and faster.

## What Changes

- Add a managed-character deletion command that removes a selected registered
  character only after an explicit confirmation and reports actionable errors.
- Prevent deletion of a character while its local runtime is active so users
  do not remove persisted data owned by a running service.
- Let `gyrognome dashboard` run without an identifier and present an
  interactive registered-character selector before opening the chosen
  dashboard.
- Preserve direct dashboard invocation with an explicit character identifier
  for scripts and users who already know the target.

## Capabilities

### New Capabilities

- `managed-character-administration`: Credential-safe selection and deletion
  of locally registered managed characters.

### Modified Capabilities

- `local-character-runtime`: Add the managed-character deletion lifecycle and
  its safety constraints to local registration and persistence behavior.
- `terminal-dashboard`: Allow dashboard entry through an interactive managed
  character selection flow when no identifier is supplied.

## Impact

Affected areas include the Clap CLI command surface, local managed-character
store and lifecycle coordination, terminal interaction code, and runtime/CLI
tests. No network behavior, browser save parsing, or external dependencies
change.
