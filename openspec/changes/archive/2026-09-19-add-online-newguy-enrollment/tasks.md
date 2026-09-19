# Tasks

## 1. Enrollment protocol and safe transport

- [x] 1.1 Add a constrained official-endpoint enrollment transport and a credential-safe create-response classifier that distinguishes success, duplicate-name rejection, and incomplete outcomes; verify focused unit tests cover each classification without retaining raw responses or signed URLs.
- [x] 1.2 Build the final draft's private online source document and canonical online metadata from a validated successful enrollment result; verify unit tests retain the passkey only in the private document and expose only realm and host in canonical state.
- [x] 1.3 Construct and deliver the browser-compatible initial `s` report from a successful enrollment result before registration; verify request-construction tests assert the trigger, field order, official endpoint, and credential-redacted outcomes.
- [x] 1.4 Gate enrollment before transport on the bundled passing enrollment-conformance evidence; verify malformed, sensitive, unavailable, and non-passing evidence causes zero create and report transport calls.

## 2. Online New Guy interaction

- [x] 2.1 Add an Offline/Online Mode row to the interactive wizard while retaining explicit traits as the offline-only scripted path; verify switching modes preserves the provisional draft, scripted offline creation remains unchanged, and no transport occurs before Online Sold!.
- [x] 2.2 Extend the existing terminal wizard with a labeled online draft that returns a validated final draft only after Sold!; verify cancellation, invalid-name correction, terminal restoration, and no-pre-Sold! transport behavior.
- [x] 2.3 Wire Sold! to one foreground enrollment activation: retain the draft after a confirmed duplicate-name rejection, then register only after create and initial `s` delivery both succeed; verify fake-transport wizard/service tests cover success and duplicate rejection.
- [x] 2.4 End ambiguous create and initial-report failures without retry, local registration, or a claim about server reservation; verify each failure path produces a credential-safe incomplete-enrollment error and leaves the managed-character list unchanged.

## 3. Boundary and integration coverage

- [x] 3.1 Add CLI integration coverage for successful online enrollment, rejected name correction, cancellation, evidence failure, and incomplete remote outcomes; verify registration output and persisted inspection redact credentials.
- [x] 3.2 Extend transport-boundary and fixture-safety tests to prove imports, offline New Guy, workers, lifecycle commands, dashboards, administration, and manual reports cannot invoke enrollment; verify no test fixture or diagnostic contains a passkey, raw response, or complete signed URL.
- [x] 3.3 Run formatting, warning-denied linting, focused enrollment tests, fixture-safety tests, and the full Rust and Node suites; verify all pass before marking the change complete.
