# Tasks

## 1. Simplify Dashboard Commands

- [x] 1.1 Remove the manual refresh command, `r` key mapping, application
  branch, and footer help from the dashboard; verify key-mapping and rendered
  help tests show `r` is no longer actionable or advertised.
- [x] 1.2 Remove the dedicated `c` recovery key mapping and footer help while
  retaining the internal recovery lifecycle action; verify dashboard input
  tests treat `c` as unused while Ctrl-C still quits.

## 2. Make Lifecycle Recovery Contextual

- [x] 2.1 Map the dashboard lifecycle toggle to stop for active services, start
  for inactive or unavailable status, and recover for failed services; verify
  state-selection tests cover every service-state case.
- [x] 2.2 Update confirmation and footer wording so the single lifecycle control
  accurately communicates recovery for a failed service; verify full and
  compact rendering tests cover normal and failed states.
- [x] 2.3 Verify a confirmed failed-state lifecycle action invokes recovery,
  refreshes displayed state after success, and preserves actionable errors on
  failure with targeted dashboard tests.

## 3. Documentation and Validation

- [x] 3.1 Update README dashboard controls to describe automatic refresh and
  contextual start, stop, or recovery while retaining the documented
  `gyro recover` CLI command; verify no dashboard documentation advertises
  manual refresh or a dedicated recovery key.
- [x] 3.2 Bump the project version from `1.0.0` to `1.1.0` in the Rust and Node
  manifests and regenerate their lockfiles; verify `Cargo.toml`, the Gyrognome
  package entry in `Cargo.lock`, `package.json`, and both root package entries
  in `package-lock.json` all report `1.1.0`.
- [x] 3.3 Run the targeted Rust dashboard and lifecycle tests, then run the
  repository's standard Rust and Node validation suites.
