## 1. Redacted enrollment observations

- [ ] 1.1 Extend the disposable Playwright harness to record credential-free `cmd=create` descriptors, browser-visible enrollment outcomes, and create/initial-`s` ordering; verify unit tests reject passkeys, response bodies, profiles, raw saves, and complete signed URLs.
- [ ] 1.2 Drive a confirmed disposable successful-enrollment scenario through the official browser; verify dry-run evidence records creation and initial-report order without transmitting either request, while a separately confirmed live run records a passing redacted result.

## 2. Failure and duplicate-name conformance

- [ ] 2.1 Add a second disposable browser context that attempts the name reserved by the successful scenario; verify the recorded browser-visible duplicate-name rejection is distinct from enrollment success and the context creates no additional online identity.
- [ ] 2.2 Add a response-interruption scenario for a disposable enrollment attempt; verify evidence records the observed failure and any browser retry behavior as unconfirmed, without inferring server-side name reservation.
- [ ] 2.3 Keep the existing confirmation gates and endpoint allowlist effective for every enrollment scenario; verify runs without the disposable confirmation issue no creation requests and live requests require the distinct live-submission confirmation.

## 3. Evidence gate and delivery

- [ ] 3.1 Define a credential-free enrollment evidence schema and validation that require successful creation, duplicate-name, initial-report-order, and interrupted-enrollment observations; verify incomplete or sensitive evidence fails validation.
- [ ] 3.2 Document the disposable-only procedure, its confirmations, the evidence fields, and the limits of interrupted-request observations; verify documentation never directs an operator to provide a real save, passkey, or character name.
- [ ] 3.3 Run formatting, warning-denied linting, fixture-safety and harness tests, the full Rust and Node test suites, plus the explicitly confirmed disposable conformance run; verify passing evidence is committed without credentials and normal CLI paths remain transport-free.
