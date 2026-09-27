# opt-in-leaderboard-reporting Specification

## Purpose

Allow an operator to deliberately submit the current state of a
browser-imported managed character while preserving Gyrognome's default
credential-safe, transport-free operation.

## Requirements

### Requirement: Confirmed one-shot managed-character reporting

The system SHALL provide a foreground command that submits one profile-compatible
manual-brag (`t=b`) report for a selected managed character. It SHALL display
the safe identity, continuation profile, and realm and require explicit
interactive confirmation for each submission. Declining SHALL send no request
and leave state unchanged. Desktop confirmation SHALL NOT override eligibility.

#### Scenario: Confirming a report submission

- **WHEN** an operator confirms an eligible character's report
- **THEN** exactly one manual-brag request uses its persisted state and
  matching browser or desktop protocol

#### Scenario: Declining a report submission

- **WHEN** an operator declines confirmation
- **THEN** no request is sent and managed state is unchanged

### Requirement: Reporting eligibility and conformance gate

The system SHALL allow manual brag, motto, and guild actions for valid imported
online browser characters with retained passkeys while active or inactive when
applicable evidence passes. Desktop characters SHALL additionally require
matching profile, import-path, realm, endpoint, credential-mode, encoding, and
operation evidence, and SHALL NOT have local-only advancement provenance.
For import-path matching, an explicitly proven deterministic load
normalization MAY match its canonical path only under the equivalence rule
defined by desktop leaderboard conformance. No unknown, progression-affecting,
or combined adaptation inherits eligibility through that rule.
Each explicit action SHALL serialize with other online actions without
stopping, restarting, or interrupting runtime ownership.

Offline origins, invalid/missing credentials, unsupported authentication or
endpoints, local-only provenance, and incomplete evidence SHALL be rejected
before transport, with a safe reason. Valid browser evidence SHALL NOT confer
desktop eligibility.

#### Scenario: Reporting an offline-originated character

- **WHEN** an operator requests manual brag, motto, or guild action without a
  valid imported online credential
- **THEN** the character is reported ineligible and no request is sent

#### Scenario: Reporting an active managed character

- **WHEN** an otherwise eligible active character receives a manual-brag request
- **THEN** one report uses a coherent persisted character/profile snapshot
  through the serialized boundary without interrupting ownership

#### Scenario: Editing the profile of an active managed character

- **WHEN** an otherwise eligible active character receives a motto or guild action
- **THEN** the action uses the serialized boundary without interrupting ownership

#### Scenario: Conformance evidence is unavailable

- **WHEN** required evidence does not validate for the requested operation
- **THEN** the operation is refused without contacting the leaderboard

#### Scenario: Classic operation not covered by evidence

- **WHEN** desktop evidence covers another realm, credential mode, import path,
  or operation
- **THEN** the requested action is ineligible even if browser evidence passes

#### Scenario: Reporting after the desktop spelling normalization

- **WHEN** a desktop import records only `load-spelling-patch`, its canonical
  and protocol equivalence is validated, and all other operation evidence
  matches
- **THEN** reporting eligibility is evaluated as the evidenced canonical path
  while the recorded adaptation remains visible

#### Scenario: Reporting after an unapproved adaptation

- **WHEN** a desktop import records a progression-affecting, unknown, or
  combined adaptation without separately matching evidence
- **THEN** the request is refused before transport even if one recorded
  adaptation would be eligible by itself

### Requirement: Credential-safe report delivery

The reporting surface SHALL construct profile-compatible manual-brag,
motto-change, and guild requests from coherent persisted state, online profile,
and retained private authentication. Browser delivery SHALL retain its existing
official endpoint behavior. Desktop delivery SHALL use only an explicitly
verified allowlisted official HTTPS endpoint and authentication contract; a
saved host or URL userinfo SHALL NOT authorize a destination. Redirects SHALL
NOT leak authentication or switch to an unapproved endpoint or HTTP.

Output SHALL contain only safe identity, operation, eligibility reason, and
categorized outcome. Logs/errors SHALL NOT contain passkeys, account
logins/passwords, signed or authenticated URLs, raw saves/responses, or browser
profiles. Successful HTTP status SHALL mean delivered, not proof of normal
leaderboard classification. Guild responses SHALL be bounded, normalized to
remove previous/submitted designations, and matched only to sanitized
profile-and-realm-specific response fingerprints. Raw and normalized text
SHALL remain internal and SHALL NOT be logged, returned, or persisted.

#### Scenario: Delivering an eligible report

- **WHEN** the approved endpoint returns successful HTTP status for manual brag
  or motto change
- **THEN** only a safe delivered outcome is reported, without a classification
  claim

#### Scenario: Categorizing a guild response

- **WHEN** a guild response matches a validated accepted, rejected, or
  indeterminate form for the character's profile and realm
- **THEN** only that safe category is exposed

#### Scenario: Guild response fingerprint is unknown

- **WHEN** a bounded guild response matches no validated accepted/rejected
  fingerprint
- **THEN** the outcome is indeterminate and neither raw nor normalized response
  text is exposed or persisted

#### Scenario: Handling delivery failure

- **WHEN** delivery fails or returns unsuccessful HTTP status
- **THEN** a safe failure preserves state not explicitly locally persistent
  under the requested action and triggers no automatic retry

#### Scenario: Save contains an unapproved endpoint

- **WHEN** a desktop save supplies an arbitrary host, URL userinfo, or an
  endpoint with no verified HTTPS mapping
- **THEN** no credentials are transmitted and online eligibility explains the
  unsupported endpoint without echoing its raw value

### Requirement: No implicit or automated reporting

The system SHALL submit profile-compatible requests only for persisted
level-up/act-completion events from an eligible active online runtime or an
operator's explicit manual-brag, motto-change, or guild-designation submission.
Empty guild means leave. Automatic delivery SHALL be one attempt, never queued
or retried. Registration, inspection, dashboard refresh, lifecycle operations,
and administration SHALL NOT independently submit requests. Ineligible desktop
events SHALL be discarded for delivery, not held for a later evidence update.

#### Scenario: Advancing a managed online character

- **WHEN** an eligible active online character persists a level-up or act event
- **THEN** exactly one corresponding report is attempted without foreground
  confirmation

#### Scenario: Advancing a managed character

- **WHEN** a managed character advances through its worker or local lifecycle
- **THEN** only persisted level/act events from an eligible online worker can
  produce reports

#### Scenario: Advancing a managed offline character

- **WHEN** an active character without an online credential advances
- **THEN** no leaderboard request is submitted

#### Scenario: Refreshing a dashboard

- **WHEN** the dashboard refreshes without an explicit online action
- **THEN** it sends no leaderboard request

#### Scenario: Gated desktop advancement

- **WHEN** an ineligible desktop character crosses a reporting boundary
- **THEN** it advances locally, records required local-only provenance, and
  neither sends nor queues the report
