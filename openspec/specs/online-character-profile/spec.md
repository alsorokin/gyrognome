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
confirmed submission SHALL attempt exactly one profile-compatible mutation.

For fingerprint-based verification, persisted guild SHALL change when the response matches an accepted fingerprint
produced by the same versioned normalization contract used for live evidence.
If the response remains indeterminate, the same explicit action MAY perform
one credential-free read of the verified realm page. The observed guild SHALL
be persisted only when the exact character is present and the observed guild
matches the submitted ASCII designation case-insensitively; the server's
canonical capitalization SHALL be retained. An observed empty membership
SHALL similarly confirm an explicit leave request.

Rejected or unconfirmed outcomes SHALL preserve the prior designation.
Unsuccessful or undeliverable outcomes SHALL likewise preserve it unless the
permitted public observation confirms desired membership. Reconciliation SHALL
NOT repeat the guild mutation. Supported Pemptus imports SHALL use the explicit
public-confirmation strategy below rather than mandatory response fingerprints.

Designations SHALL accept non-control Unicode text including empty text. The
initial desktop ASCII contract SHALL reject non-ASCII input before state
change/transport; browser Unicode behavior SHALL remain unchanged. Cancelling
SHALL preserve state and send nothing.

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
profile/realm/operation readiness, an approved official endpoint, and eligible
advancement provenance. Offline characters, invalid credentials, unsupported
endpoints or authentication, invalid/incomplete evidence, and local-only
desktop imports SHALL be refused before transport or profile mutation.
Supported Pemptus guild readiness SHALL use its explicit public-confirmation
contract without requiring historical guild-response fingerprints.
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

### Requirement: Evidence-scoped Pemptus profile actions

Eligible Pemptus desktop imports SHALL support motto set/clear and guild
join/change/leave through their fixed HTTPS passkey-only contract. Motto and
guild eligibility SHALL be independent of each other and of manual/automatic
reporting evidence, subject to the existing import, encoding, and provenance
restrictions.

Unadapted, spelling-only, and exact indexed quest-placeholder-only imports SHALL
share approved motto readiness without relabeling actual evidence. Supported
Pemptus guild actions SHALL use explicit public confirmation instead of requiring
historical response fingerprints. Spoltog fingerprints SHALL NOT classify
Pemptus responses, and Spoltog's existing verification SHALL remain unchanged.

A guild action SHALL attempt at most one native mutation, then perform one
bounded credential-free observation of the exact character in the Pemptus realm.
Desired membership SHALL match ASCII-case-insensitively and retain observed
canonical spelling; leave SHALL require authoritative no-guild state. HTTP
success or an empty body alone SHALL NOT confirm membership. A confirmed desired
state SHALL NOT be described as uniquely attributable request consumption when
the state already existed. Missing, malformed, ambiguous, conflicting, or failed
observations SHALL preserve prior membership and produce a safe non-success
outcome. A delivery error SHALL NOT authorize retry and MAY be reconciled against
the same public state.

Motto persistence, confirmed-only guild persistence, serialization with worker
reports, safe outcomes, and no-retry behavior SHALL remain unchanged. Unsupported
or combined adaptations and permanent local-only histories SHALL remain excluded.

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
