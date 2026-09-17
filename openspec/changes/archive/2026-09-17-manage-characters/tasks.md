## 1. Character removal persistence

- [x] 1.1 Extend the managed-character store with an atomic single-character removal operation that removes state, original save data, and runtime metadata together; verify storage tests cover successful removal, unknown identifiers, and rollback after an injected persistence failure.
- [x] 1.2 Coordinate removal with the existing runtime ownership mechanism so an owned character cannot be removed; verify a worker/lock test proves that a rejected removal preserves the running character and its records.

## 2. Character-administration CLI

- [x] 2.1 Add the `gyrognome delete <id>` command, resolve and display only the target's credential-safe identity, and require explicit standard-input confirmation; verify CLI tests cover confirmation, cancellation, malformed identifiers, and unknown characters.
- [x] 2.2 Integrate deletion with lifecycle status so active characters return an actionable stop-first error without mutation; verify a CLI or integration test covers deletion attempted against an active service/owner.

## 3. Dashboard character selection

- [x] 3.1 Make the dashboard identifier optional while preserving direct identifier-based dashboard startup; verify command parsing and existing direct-dashboard tests continue to pass.
- [x] 3.2 Implement a terminal-safe, keyboard-navigable registered-character selector that returns a selected identifier or cancellation and displays only credential-safe identity; verify selector tests cover navigation, selection, cancellation, and terminal cleanup on failure.
- [x] 3.3 Handle an empty registration list before entering the dashboard and route a selected identifier into the existing dashboard loop; verify integration tests cover empty state, selected-character startup, and no persisted-state mutation on cancellation.

## 4. Validation

- [x] 4.1 Run the focused Rust CLI, runtime-boundary, local-character-runtime, and dashboard test suites; verify the new management flows and existing lifecycle/dashboard behavior pass.
- [x] 4.2 Run `openspec validate manage-characters --strict`; verify all proposal, design, specification, and task artifacts are valid.
