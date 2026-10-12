# Tasks

## 1. Detached launcher

- [x] 1.1 In `src/reporting.rs`, replace the `.status()` call with a detached spawn of `xdg-open` (null stdio, `process_group(0)`) plus a background thread that waits up to 3 s and reports through an `mpsc` receiver, then keeps waiting to reap the child; verify with unit tests of the wait helper using `true` (ok), `false` (failure), and a long `sleep` (ok after timeout, returns within ~3 s)
- [x] 1.2 Update the manual-brag opener seam to return the pending receiver and remove the shared `eprintln!`; verify existing reporting tests (including the opener failure test) pass after adaptation

## 2. Callers

- [x] 2.1 CLI `report`: wait on the receiver and print the existing stderr failure message; verify `cargo test` CLI/reporting tests pass
- [x] 2.2 Dashboard Brag: keep the pending receiver in `DashboardState`, poll it with a capped poll timeout, and show an opening failure as the action message; verify with a dashboard test that a failing fake opener produces the message and the delivery message is unchanged

## 3. Check

- [x] 3.1 Manually: close the browser, brag from the dashboard, and confirm no log output appears in the terminal and the dashboard stays responsive
