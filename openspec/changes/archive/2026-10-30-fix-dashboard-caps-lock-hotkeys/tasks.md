# Tasks

## 1. Case-insensitive keyboard dispatch

- [x] 1.1 Update `command` and `selection_command` in `src/dashboard.rs` to match ASCII letter shortcuts case-insensitively, including Ctrl+C. Keep matching local to command interpretation rather than changing incoming text globally; preserve existing modifiers, event-kind filters, and action dispatch. Extend nearby key-mapping tests to cover every lowercase/uppercase shortcut pair and unchanged non-letter controls; verify with `cargo test dashboard::tests`.
- [x] 1.2 Update only Ctrl+C recognition in `editor_command` to accept either case of `c`, leaving printable text unchanged. Extend existing profile-editor tests for uppercase shortcut text, mixed-case submission, and uppercase Ctrl+C without submission; verify with `cargo test dashboard::tests`.
- [x] 1.3 Add a short note beneath README dashboard controls that letter shortcuts accept either case and Caps Lock does not alter editor text handling. Verify the wording matches the implemented behavior and run `cargo fmt --check`.

## 2. Interactive acceptance

- [x] 2.1 In a terminal with Caps Lock enabled, check selector J/K/Q, dashboard letter actions, confirmation/cancellation for lifecycle and autostart, and mixed-case motto/guild editing without submitting live requests or changing service configuration. Confirm quit restores the terminal and report any terminal-specific limitation rather than claiming unperformed checks.

User confirmed the interactive fix works on 2026-10-30.
