# Spec Delta

## ADDED Requirements

### Requirement: Evidence-scoped Pemptus profile actions

Eligible Pemptus desktop imports SHALL support motto set/clear and guild
join/change/leave through their fixed HTTPS passkey-only contract. Motto and
guild eligibility SHALL be independent of each other and of manual/automatic
reporting evidence, subject to the existing import, encoding, and provenance
restrictions.

Guild classification SHALL require matching Pemptus guild evidence, including
the supported response-normalization contract and accepted/rejected outcomes.
Spoltog guild fingerprints SHALL NOT classify Pemptus responses. Ambiguous,
unknown, or unconfirmed responses SHALL NOT commit an optimistic guild change.
Any public reconciliation SHALL use the matching realm and exact character
identity. Motto persistence, accepted-only guild persistence, serialization
with worker reports, safe outcomes, and no-retry behavior SHALL remain
unchanged.

#### Scenario: Changing or clearing a Pemptus motto

- **WHEN** an operator submits a valid motto, including an empty clear, with
  passing Pemptus motto evidence
- **THEN** the motto is persisted under the existing profile-action contract
  and one Pemptus request is attempted without account authorization

#### Scenario: Missing guild evidence while motto is supported

- **WHEN** Pemptus motto evidence passes but Pemptus guild evidence is absent
- **THEN** motto actions remain available and a guild action is rejected
  before transport or guild persistence

#### Scenario: Joining, changing, or leaving a Pemptus guild

- **WHEN** a supported guild designation or empty leave is submitted with
  matching passing Pemptus guild evidence
- **THEN** one realm-compatible request is attempted and the persisted guild
  changes only after an accepted outcome is established

#### Scenario: Rejecting another realm's guild evidence

- **WHEN** only Spoltog guild fingerprints are available for a Pemptus action
- **THEN** they do not enable the action or classify its response

#### Scenario: Failing to confirm a guild result

- **WHEN** a Pemptus guild response is rejected, ambiguous, unknown, or cannot
  be reconciled against the exact character on the Pemptus public page
- **THEN** the previous persisted guild remains intact and a safe categorized
  failure is reported without a retry

#### Scenario: Serializing a profile change with progression

- **WHEN** an eligible Pemptus profile action contends with a worker report
- **THEN** the existing online-action serialization determines request order
  and the later snapshot uses the profile state established before it
