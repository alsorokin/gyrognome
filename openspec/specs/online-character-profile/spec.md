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
character's motto while active or inactive. Each confirmed action SHALL persist
the selected motto and attempt exactly one profile-compatible motto-change
(`t=m`) report. Later reports SHALL use the persisted motto. Cancelling SHALL
preserve state and send nothing.

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

#### Scenario: Rejecting control characters in a motto

- **WHEN** an operator submits motto text containing control characters
- **THEN** it is rejected before persistence or transport

#### Scenario: Preserving browser Unicode motto support

- **WHEN** an eligible browser character receives non-control Unicode motto
  input
- **THEN** the input remains supported without transliteration

### Requirement: Explicit guild membership management

CLI and dashboard SHALL accept guild designations for eligible online
characters while active or inactive. Non-empty input SHALL request
joining/changing guild; empty input SHALL request leaving. Each confirmed
submission SHALL attempt one profile-compatible mutation. Only accepted
fingerprints under the shared versioned normalization contract or permitted
public confirmation SHALL change persisted membership. Non-control Unicode
SHALL be accepted except under the desktop ASCII contract.

#### Scenario: Joining a guild

- **WHEN** a valid designation receives an acceptance response recognized by
  the shared evidence fingerprint contract
- **THEN** it is persisted and a safe accepted outcome is reported

#### Scenario: Submitting an empty guild name

- **WHEN** empty input receives a recognized acceptance response or an
  indeterminate response followed by a public observation of the exact
  character without a guild
- **THEN** the persisted designation is cleared, a safe accepted outcome is
  reported, and the mutation is not repeated

#### Scenario: Guild request is rejected or indeterminate

- **WHEN** the endpoint rejects a request, delivery fails, or neither the
  response fingerprint nor the bounded public observation verifies the
  submitted membership
- **THEN** a safe failure category preserves the prior guild without retrying
  the mutation

#### Scenario: Guild response is indeterminate but membership changed

- **WHEN** an explicit guild mutation has an indeterminate response and one
  credential-free read of the verified realm page shows the exact character in
  a guild matching the submitted ASCII designation case-insensitively
- **THEN** the observed canonical guild designation is persisted and the
  mutation is not repeated

#### Scenario: Cancelling dashboard guild input

- **WHEN** an operator cancels the editor
- **THEN** guild state is unchanged and no request occurs

#### Scenario: Wrong-profile guild evidence

- **WHEN** a desktop guild operation has only browser response fingerprints
  and no supported public-confirmation strategy
- **THEN** it is refused before delivery rather than using those fingerprints

#### Scenario: Rejecting unsupported guild designations

- **WHEN** an operator submits control characters for any character or
  non-ASCII input for a desktop character
- **THEN** the designation is rejected before persistence or transport

#### Scenario: Preserving browser Unicode guild support

- **WHEN** an eligible browser character receives a non-control Unicode guild
  designation
- **THEN** the designation remains supported without transliteration

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

Before transport or profile mutation, online profile actions SHALL require
valid retained authentication, passing profile/realm/operation readiness, an
approved official endpoint, and eligible advancement provenance. Supported
Pemptus guild actions SHALL use explicit public confirmation. Output and
diagnostics SHALL exclude credentials and other sensitive profile data.

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

#### Scenario: Refusing unsupported profile actions

- **WHEN** credentials are invalid, authentication or endpoint is unsupported,
  readiness fails, or advancement provenance is ineligible
- **THEN** the action is refused before transport or profile mutation

#### Scenario: Keeping sensitive data out of diagnostics

- **WHEN** a profile action succeeds, is rejected, or fails
- **THEN** output excludes logins, passwords, passkeys, signed or authenticated
  URLs, raw saves or responses, and browser profiles

### Requirement: Evidence-scoped Pemptus profile actions

Eligible Pemptus desktop imports SHALL support motto and guild actions under
their fixed HTTPS passkey-only contract and existing import, encoding, and
provenance restrictions. Motto and guild eligibility SHALL be independent.
Pemptus guild actions SHALL require public confirmation, not response
fingerprints.

#### Scenario: Changing or clearing a Pemptus motto

- **WHEN** an operator submits a valid motto, including an empty clear, with
  passing Pemptus motto evidence
- **THEN** the motto is persisted under the existing profile-action contract
  and one Pemptus request is attempted without account authorization

#### Scenario: Using supported actions without historical guild fingerprints

- **WHEN** a supported Pemptus import has valid motto coverage but no retained
  guild-response fingerprint record
- **THEN** motto and publicly confirmed guild actions remain available without
  inventing fingerprints or enabling evidence

#### Scenario: Applying independent Pemptus evidence

- **WHEN** an unadapted, spelling-only, or exact indexed quest-placeholder-only
  import is eligible for a Pemptus profile action
- **THEN** approved motto readiness is shared without relabeling evidence, and
  motto and guild eligibility remain independent of manual or automatic
  reporting evidence

#### Scenario: Excluding unsupported Pemptus imports

- **WHEN** an import uses an unsupported or combined adaptation, or is a
  permanent local-only history
- **THEN** Pemptus profile actions remain unavailable

#### Scenario: Joining, changing, or leaving a Pemptus guild

- **WHEN** a supported guild designation or empty leave is submitted with
  the supported Pemptus public-confirmation strategy
- **THEN** one realm-compatible request is attempted and the persisted guild
  changes only after the desired public membership is confirmed

#### Scenario: Preserving realm-specific guild verification

- **WHEN** only Spoltog guild fingerprints are available for a Pemptus action
- **THEN** they do not classify its response and its result still requires
  Pemptus public confirmation

#### Scenario: Failing to confirm a guild result

- **WHEN** a Pemptus guild response is rejected, ambiguous, unknown, or cannot
  be reconciled against the exact character on the Pemptus public page
- **THEN** the previous persisted guild remains intact and a safe categorized
  failure is reported without a retry

#### Scenario: Confirming Pemptus guild membership publicly

- **WHEN** one Pemptus guild mutation is followed by a bounded,
  credential-free observation of the exact character
- **THEN** a matching designation is confirmed ASCII-case-insensitively with
  observed spelling retained, and leave requires authoritative no-guild state;
  HTTP success or an empty response alone is insufficient

#### Scenario: Handling uncertain Pemptus guild outcomes

- **WHEN** public guild observation is missing, malformed, ambiguous,
  conflicting, or fails, or delivery of the sole mutation fails
- **THEN** prior membership is preserved unless that same observation confirms
  the desired state, and the mutation is never retried

#### Scenario: Avoiding claims about pre-existing Pemptus membership

- **WHEN** public observation confirms a desired guild state that already
  existed
- **THEN** the result is not described as uniquely attributable request
  consumption

#### Scenario: Confirming canonical public guild spelling

- **WHEN** one Pemptus request asks for `BEERGuild` and the exact character's
  public row confirms linked membership as `BEERguild`
- **THEN** the persisted guild is `BEERguild` without a second mutation

#### Scenario: Rejecting an invalid designation without optimistic persistence

- **WHEN** an invalid guild designation is submitted and the bounded public
  observation does not confirm that requested membership
- **THEN** the previous guild stays intact and a safe non-success outcome is
  reported without claiming a missing rejection fingerprint

#### Scenario: Reconciling uncertain delivery once

- **WHEN** guild delivery fails after the sole attempt but the bounded public
  observation confirms the requested membership
- **THEN** membership can be persisted without retrying the mutation

#### Scenario: Confirming a leave with a declared empty trailing column

- **WHEN** the Pemptus table explicitly declares Guild and the unique exact-name
  row unambiguously omits only that empty trailing cell
- **THEN** leave can be confirmed, while absent headers, malformed alignment,
  duplicate rows, and conflicting content remain unknown

#### Scenario: Serializing a profile change with progression

- **WHEN** an eligible Pemptus profile action contends with a worker report
- **THEN** the existing online-action serialization determines request order
  and the later snapshot uses the profile state established before it
