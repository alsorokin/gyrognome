# online-character-profile Specification

## Purpose

Provide durable, credential-safe management of an online character's motto and
guild membership through explicit client actions that remain available while
the local runtime is active.

## Requirements

### Requirement: Persistent online profile metadata

The system SHALL retain each managed online character's motto and guild as
safe profile metadata separate from deterministic simulation state.
Registration SHALL initialize them from optional browser fields or supported
desktop profile properties; evidenced missing defaults SHALL be empty.
Account credentials SHALL NOT be treated as public profile metadata.
Updates SHALL NOT overwrite concurrent simulation progress.

#### Scenario: Importing existing profile values

- **WHEN** an online browser save contains motto or guild fields
- **THEN** registration retains them for inspection, editing, and requests

#### Scenario: Importing a save without profile values

- **WHEN** a browser save omits motto or guild
- **THEN** empty values are used without rejecting the save

#### Scenario: Updating a running character

- **WHEN** a profile action and runtime advancement persist concurrently
- **THEN** both complete updates remain durable without overwriting each other

#### Scenario: Importing desktop profile properties

- **WHEN** a supported desktop save contains motto and guild values
- **THEN** they initialize safe profile metadata, while account/password
  properties remain private

### Requirement: Explicit motto management

CLI and dashboard SHALL let an operator set or clear an eligible online
character's motto while active or inactive. The action SHALL persist the motto
and attempt exactly one profile-compatible motto-change (`t=m`) report. The
selected motto SHALL remain after delivery failure and be used by later
reports. Cancelling SHALL preserve state and send nothing.

Input SHALL accept non-control Unicode text, including empty text, and reject
controls before persistence/transport. For the initial desktop ASCII contract,
non-ASCII input SHALL additionally fail with an unsupported-encoding error
before changing state or sending a request, rather than being transliterated.
Browser Unicode support SHALL remain unchanged.

#### Scenario: Setting a motto

- **WHEN** an eligible character receives a valid representable motto
- **THEN** it is persisted, one motto report is attempted, and later reports
  use it

#### Scenario: Clearing a motto

- **WHEN** an eligible character receives an explicitly empty motto
- **THEN** the empty value is persisted and one report has an empty motto field

#### Scenario: Motto delivery fails

- **WHEN** delivery of a persisted motto fails or is rejected
- **THEN** the selected motto remains, a safe failure is reported, and no retry
  occurs

#### Scenario: Cancelling dashboard motto input

- **WHEN** an operator cancels the editor
- **THEN** the previous motto remains and no request occurs

#### Scenario: Unrepresentable desktop motto

- **WHEN** a desktop character receives non-ASCII motto input
- **THEN** neither local state nor server state is changed and a safe encoding
  error is shown

### Requirement: Explicit guild membership management

CLI and dashboard SHALL let an operator submit a guild designation for an
eligible online character while active or inactive. Non-empty input SHALL
request joining/changing guild; empty input SHALL request leaving. Each
confirmed submission SHALL attempt exactly one profile-compatible request.
Persisted guild SHALL change only for a verified accepted normalized response
fingerprint for the profile, realm, and submitted operation. Rejected, unknown,
oversized, unsuccessful, or undeliverable responses SHALL preserve the prior
designation.

Designations SHALL accept non-control Unicode text including empty text.
The initial desktop ASCII contract SHALL reject non-ASCII input before
state change/transport; browser Unicode behavior SHALL remain unchanged.
Cancelling SHALL preserve state and send nothing.

#### Scenario: Joining a guild

- **WHEN** a valid designation receives evidence-backed acceptance
- **THEN** it is persisted and a safe accepted outcome is reported

#### Scenario: Submitting an empty guild name

- **WHEN** empty input receives evidence-backed acceptance
- **THEN** the persisted designation is cleared and a safe accepted outcome
  is reported

#### Scenario: Guild request is rejected or indeterminate

- **WHEN** the endpoint rejects a request, returns an unrecognized response,
  or cannot be reached
- **THEN** a safe failure category preserves the prior guild without retry

#### Scenario: Cancelling dashboard guild input

- **WHEN** an operator cancels the editor
- **THEN** guild state is unchanged and no request occurs

#### Scenario: Wrong-profile guild evidence

- **WHEN** a desktop guild operation has only browser response fingerprints
- **THEN** it is refused before delivery rather than using those fingerprints

### Requirement: Live online-action ordering

The system SHALL serialize a managed character's explicit motto changes, guild
actions, manual brags, and worker-generated reports through a short-lived
per-character online-action boundary. This boundary SHALL NOT transfer or
interrupt runtime ownership. Each emitted request SHALL use one coherent
persisted character and profile snapshot, and requests SHALL be attempted in
the order in which they acquire that boundary.

#### Scenario: Changing a motto during automatic reporting

- **WHEN** a motto action and a worker-generated report occur concurrently
- **THEN** the system sends them in a defined order and the later request uses
  the profile state established by the earlier completed action

#### Scenario: Editing a profile while the runtime owns the character

- **WHEN** an operator changes a motto or guild while the worker owns the
  character's simulation lock
- **THEN** the profile action completes without stopping the runtime or
  violating exclusive simulation ownership

### Requirement: Credential-safe profile actions

Online profile actions SHALL require valid retained authentication, passing
profile/realm/operation conformance, an approved official endpoint, and eligible
advancement provenance. Offline characters, invalid credentials, unsupported
endpoints or authentication, invalid/incomplete evidence, and local-only
desktop imports SHALL be refused before transport or profile mutation.
Output/diagnostics SHALL NOT expose account logins/passwords, passkeys, signed
or authenticated URLs, raw saves/responses, or browser profiles.

#### Scenario: Editing an offline character

- **WHEN** an offline character receives a motto or guild action
- **THEN** it is reported ineligible and no request occurs

#### Scenario: Profile conformance evidence is unavailable

- **WHEN** required evidence is invalid or incomplete
- **THEN** the action is refused without contacting the endpoint

#### Scenario: Reporting a profile action outcome

- **WHEN** a request completes, is rejected, or fails
- **THEN** only a safe categorized outcome is exposed

#### Scenario: Editing a local-only desktop import

- **WHEN** an import advanced while reporting was gated receives a profile action
- **THEN** the action is refused without persistence or network activity and
  the need for a fresh official-client import is explained
