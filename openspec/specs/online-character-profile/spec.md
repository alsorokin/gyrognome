# online-character-profile Specification

## Purpose

Provide durable, credential-safe management of an online character's motto and
guild membership through explicit client actions that remain available while
the local runtime is active.

## Requirements

### Requirement: Persistent online profile metadata

The system SHALL retain each managed online character's motto and guild
designation as credential-safe profile metadata separate from deterministic
simulation state. Registration SHALL initialize those values from optional
browser-save fields, defaulting each missing field to an empty value. Profile
updates SHALL NOT overwrite concurrently persisted simulation progress.

#### Scenario: Importing existing profile values

- **WHEN** a user registers an online browser save containing motto or guild
  fields
- **THEN** the managed character retains those values for inspection, editing,
  and future online requests

#### Scenario: Importing a save without profile values

- **WHEN** a user registers a browser save that omits motto or guild
- **THEN** the managed character uses empty profile values without rejecting
  the save

#### Scenario: Updating a running character

- **WHEN** an explicit profile action updates a character while its runtime
  persists simulation progress
- **THEN** both the profile update and the complete simulation update remain
  durable without one overwriting the other
### Requirement: Explicit motto management

The system SHALL provide CLI and dashboard actions that let an operator set or
clear the motto of an eligible managed online character while its runtime is
active or inactive. The action SHALL persist the selected motto and submit
exactly one browser-compatible motto-change (`t=m`) report using that motto.
The persisted motto SHALL remain selected if delivery fails and SHALL be used
by later manual and automatic reports. Cancelling dashboard input SHALL leave
the motto unchanged and send no request.

Motto input SHALL accept non-control Unicode text, including an empty value for
clearing the motto, and SHALL reject control characters before changing state
or contacting the endpoint.

#### Scenario: Setting a motto

- **WHEN** an operator supplies a valid motto for an eligible managed online
  character
- **THEN** the system persists it, submits one motto-change report, and uses it
  in subsequent reports

#### Scenario: Clearing a motto

- **WHEN** an operator explicitly supplies an empty motto
- **THEN** the system persists an empty value and submits one motto-change
  report with an empty motto field

#### Scenario: Motto delivery fails

- **WHEN** a valid motto is persisted but its one-shot motto-change report
  cannot be delivered or is rejected
- **THEN** the system reports a safe failure, retains the selected motto, and
  does not retry automatically

#### Scenario: Cancelling dashboard motto input

- **WHEN** an operator cancels the dashboard motto editor
- **THEN** the previous motto remains persisted and no network request occurs
### Requirement: Explicit guild membership management

The system SHALL provide CLI and dashboard actions that let an operator submit
a guild designation while an eligible managed online character's runtime is
active or inactive. A non-empty designation SHALL request joining or changing
guilds, and an empty designation SHALL request leaving the current guild. Each
confirmed submission SHALL send exactly one browser-compatible guild request.
The system SHALL update the persisted guild designation only when the official
endpoint response matches a browser-derived accepted normalized response
fingerprint for the submitted value; a validated rejected fingerprint or an
unknown, oversized, unsuccessful, or undeliverable response SHALL preserve the
previous designation.

Guild designations SHALL accept non-control Unicode text, including an empty
value. Cancelling dashboard input SHALL leave the guild unchanged and send no
request.

#### Scenario: Joining a guild

- **WHEN** an operator supplies a valid designation and the official endpoint
  confirms the join according to the conformance evidence
- **THEN** the system persists that designation and reports a credential-safe
  accepted outcome

#### Scenario: Submitting an empty guild name

- **WHEN** an operator submits an empty guild designation and the official
  endpoint confirms it according to the conformance evidence
- **THEN** the system clears the persisted guild designation and reports a
  credential-safe accepted outcome

#### Scenario: Guild request is rejected or indeterminate

- **WHEN** the guild endpoint rejects the action, returns an unrecognized
  response, or cannot be reached
- **THEN** the system reports a safe failure category, preserves the previous
  guild designation, and does not retry automatically

#### Scenario: Cancelling dashboard guild input

- **WHEN** an operator cancels the dashboard guild editor
- **THEN** the previous guild designation remains persisted and no network
  request occurs
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

Online profile actions SHALL require a managed online character with a valid
retained credential, passing applicable conformance evidence, and the official
Progress Quest endpoint. They SHALL reject offline characters, invalid
credentials, unofficial endpoints, and invalid or incomplete evidence before
network delivery. Output and diagnostics SHALL NOT expose passkeys, complete
signed URLs, raw save documents, raw endpoint response bodies, or browser
profiles.

#### Scenario: Editing an offline character

- **WHEN** an operator requests a motto or guild action for an offline managed
  character
- **THEN** the system reports that the character is ineligible and sends no
  request

#### Scenario: Profile conformance evidence is unavailable

- **WHEN** the applicable bundled conformance evidence is incomplete or
  invalid
- **THEN** the system refuses the profile action without contacting the
  official endpoint

#### Scenario: Reporting a profile action outcome

- **WHEN** a profile request completes, is rejected, or fails
- **THEN** the system exposes only a credential-safe categorized outcome
