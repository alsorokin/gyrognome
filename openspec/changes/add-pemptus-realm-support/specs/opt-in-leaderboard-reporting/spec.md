# Spec Delta

## ADDED Requirements

### Requirement: Evidence-scoped Pemptus desktop reporting

The system SHALL support reporting from managed Pemptus desktop-6.4.4 imports
only when matching current passing evidence covers the requested operation,
import/adaptation path, encoding, and authentication contract. The Pemptus
contract SHALL require a valid passkey, empty account/password fields, the exact
realm `Pemptus`, and the exact saved endpoint
`http://progressquest.com/pemptus.php?`, delivered to
`https://progressquest.com/pemptus.php` without an Authorization header.

Manual brag, automatic level reporting, and automatic act reporting SHALL be
gated independently. Inspection and runtime SHALL agree on those gates for
equivalent inputs. Neither an endpoint mapping nor evidence for another realm,
credential mode, or operation SHALL confer eligibility. Unsupported
destinations, credentials, and encoding SHALL fail before transport.

Pemptus reporting SHALL retain the existing persistence, serialization,
no-retry, callback timing, and durable local-only provenance contracts.
Advancement with either automatic progress operation gated SHALL permanently
mark the managed timeline local-only; missing only manual or profile-action
evidence SHALL NOT do so. Browser reporting and valid Spoltog
account/password reporting SHALL retain their existing behavior.

#### Scenario: Sending an eligible Pemptus manual brag

- **WHEN** an operator requests manual brag for a fresh supported Pemptus
  managed import with matching manual-operation evidence
- **THEN** the system attempts one desktop-compatible report to the fixed
  Pemptus HTTPS endpoint without an Authorization header

#### Scenario: Missing unrelated operation evidence

- **WHEN** a supported Pemptus import has passing manual-operation evidence
  but lacks guild or automatic-operation evidence and has not advanced
- **THEN** manual brag remains eligible and the missing operations remain
  individually gated

#### Scenario: Reporting persisted Pemptus progress

- **WHEN** an eligible Pemptus runtime persists a level or act transition
- **THEN** it attempts the corresponding report after persistence using the
  current serialized profile, without a retry or deferred report queue

#### Scenario: Rejecting a mismatched Pemptus contract

- **WHEN** a Pemptus request uses a different saved endpoint, non-empty account
  or password, unsupported encoding, or mismatched evidence
- **THEN** the requested action is blocked with a credential-safe reason
  before an HTTP request is attempted

#### Scenario: Advancing before progress evidence is complete

- **WHEN** a Pemptus managed import first advances while level or act reporting
  is gated
- **THEN** advancement and local-only provenance are persisted atomically and
  later passing evidence cannot reconnect that timeline

#### Scenario: Advancing without guild evidence

- **WHEN** all other operations, including level and act reporting, are eligible
  but guild evidence is absent
- **THEN** the runtime retains online provenance and only guild actions are
  blocked by the missing guild evidence

#### Scenario: Preserving existing reporting paths

- **WHEN** browser imports or Spoltog account/password imports use their
  currently valid supported contracts
- **THEN** request construction, authentication, explicit-action behavior, and
  automatic-report ordering remain unchanged
