## 1. Selector Metadata Model

- [x] 1.1 Add a selector entry type that carries character id, credential-safe identity, last-access timestamp, and active-state metadata, and verify existing selector navigation tests still pass.
- [x] 1.2 Format the last-access timestamp and active-state label for terminal display without exposing save contents or credentials, and verify rendered selector output contains the new metadata and omits sensitive fixture values.

## 2. Data Collection and Ordering

- [x] 2.1 Update the no-id `dashboard` command path to build selector entries from registered managed characters using their persisted last-access/update timestamp, and verify the selected character id still opens the dashboard.
- [x] 2.2 Query existing local lifecycle status for each selector entry before entering terminal mode, mapping active services to active metadata and status lookup failures to a credential-safe unavailable indicator, and verify one failing status lookup does not prevent selection.
- [x] 2.3 Sort selector entries by last accessed time descending with deterministic tie handling, and verify a test list renders or selects entries in newest-first order.

## 3. Validation

- [x] 3.1 Add or update dashboard unit tests covering selector rendering of last accessed time, active/inactive/unavailable state, credential safety, and newest-first ordering.
- [x] 3.2 Add or update CLI/runtime integration coverage for `gyrognome dashboard` without an id to verify registration ordering and no-registration behavior remain correct.
- [x] 3.3 Run focused Rust tests for dashboard, lifecycle/runtime selector support, and local character runtime behavior in WSL, and verify they pass.
