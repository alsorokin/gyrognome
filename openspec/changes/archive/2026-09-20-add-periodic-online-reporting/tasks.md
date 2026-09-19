# Tasks

## 1. Trace-aware best-effort worker reporting

- [x] 1.1 Extend reporting with a credential-safe event-delivery operation that constructs official level-up and act-completion reports from a managed online character; verify request-construction tests cover browser trigger order, official endpoint pinning, and redacted outcomes.
- [x] 1.2 Update worker advancement to persist the simulation trace state before attempting each emitted online report once; verify worker tests cover persisted state before delivery, multiple events in trace order, offline suppression, and no transport before a successful persistence.
- [x] 1.3 Treat automatic delivery failures as non-fatal and non-retryable; verify fake-transport tests retain advancement, make exactly one attempted delivery per event, and leave no queued work after rejection or failure.

## 2. Dashboard manual brag

- [x] 2.1 Add an immediate documented dashboard Brag action that invokes the existing manual-reporting path for the selected character; verify dashboard interaction tests cover its key binding, safe delivered/rejected/failed status messages, and no confirmation overlay.
- [x] 2.2 Preserve dashboard credential safety and unrelated refresh/lifecycle behavior; verify eligible and ineligible brag outcomes never expose credentials or request data, while refresh, navigation, and lifecycle actions send no reports.

## 3. Boundary, CLI, and documentation coverage

- [x] 3.1 Update runtime/reporting/dashboard boundary tests and CLI integration coverage for automatic event reports, dashboard brag, offline suppression, and best-effort failures; verify only the selected worker events and explicit dashboard action can deliver reports.
- [x] 3.2 Update CLI help and roadmap documentation to describe automatic online event reporting, best-effort loss semantics, and the dashboard Brag action; verify documented behavior matches the implemented controls.
- [x] 3.3 Run formatting, warning-denied linting, focused reporting/worker/dashboard tests, and the full Rust and Node suites; verify all pass before marking the change complete.
