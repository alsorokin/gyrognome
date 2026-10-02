# Spec Delta

## ADDED Requirements

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
