# Spec Delta

## ADDED Requirements

### Requirement: Evidence-scoped Pemptus desktop reporting

The system SHALL support reporting from managed Pemptus desktop-6.4.4 imports
when current passing evidence covers the requested operation through exact or
explicitly approved equivalent import coverage, encoding, and authentication
contract. The Pemptus
contract SHALL require a valid passkey, empty account/password fields, the exact
realm `Pemptus`, and the exact saved endpoint
`http://progressquest.com/pemptus.php?`, delivered to
`https://progressquest.com/pemptus.php` without an Authorization header.

Manual brag, automatic level reporting, and automatic act reporting SHALL be
gated independently. Inspection and runtime SHALL agree on those gates for
equivalent inputs. Neither an endpoint mapping nor evidence for another realm,
credential mode, or operation SHALL confer eligibility. Unsupported
destinations, credentials, and encoding SHALL fail before transport.

Unadapted, load-spelling-only, and exact indexed quest-placeholder-only Pemptus
imports SHALL share the explicitly approved readiness family for automatic and
manual reporting. Inspection, runtime, and requested-operation callers SHALL
agree, including previously imported unadvanced records. Exact matching records
SHALL take precedence; stale, invalid, inconclusive, or duplicate matching
records SHALL NOT be bypassed with equivalent coverage. Selected evidence SHALL
retain its true observed adaptation path. Original import provenance SHALL
remain authoritative after existing runtime marker resolution. Unsupported
or combined adaptations SHALL remain excluded.

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

#### Scenario: Advancing independently of guild verification

- **WHEN** all other operations, including level and act reporting, are eligible
  but a guild action's public confirmation fails
- **THEN** the runtime retains online provenance and automatic reporting remains
  available without treating the guild result as an automatic-evidence failure

#### Scenario: Preserving existing reporting paths

- **WHEN** browser imports or Spoltog account/password imports use their
  currently valid supported contracts
- **THEN** request construction, authentication, explicit-action behavior, and
  automatic-report ordering remain unchanged

#### Scenario: Enabling automatic reporting across the supported family

- **WHEN** current valid placeholder level/act records cover a supported
  unadapted, spelling-only, or placeholder-only Pemptus import
- **THEN** inspection and runtime agree that automatic reporting is eligible
  without changing the records' declared observation paths

#### Scenario: Reusing manual and motto coverage for supported placeholder imports

- **WHEN** current valid unadapted Pemptus manual/motto records cover an otherwise
  eligible placeholder-only import through the approved policy
- **THEN** manual and motto actions are available without relabeling those records

#### Scenario: Supporting an existing unadvanced Izot-equivalent import

- **WHEN** an existing unadapted or spelling-only Pemptus managed import with
  valid credentials and no local-only history receives the revised readiness policy
- **THEN** it becomes eligible without reimport, and normal advancement does not
  fork local-only solely because its original path differs from automatic evidence

#### Scenario: Retaining adapted-import provenance after marker resolution

- **WHEN** an eligible placeholder-only runtime resolves its indexed marker
  during ordinary task completion
- **THEN** later eligibility still uses its recorded placeholder import path,
  not an invented unadapted history

#### Scenario: Keeping combined adaptations outside placeholder coverage

- **WHEN** an import combines quest-placeholder with spelling correction or
  legacy prologue despite a passing placeholder automatic record
- **THEN** the combined path remains ineligible before transport

#### Scenario: Preserving local-only permanence when placeholder coverage changes

- **WHEN** a placeholder-only managed import advances without either automatic
  record and matching records are installed afterward
- **THEN** its local-only provenance remains permanent and all online actions
  stay blocked
