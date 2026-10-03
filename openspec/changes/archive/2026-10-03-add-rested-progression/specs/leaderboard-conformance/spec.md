# Spec Delta

## ADDED Requirements

### Requirement: Rested runtime pacing readiness exception

Managed production runtimes SHALL apply rested progression to otherwise eligible online characters without requiring new live leaderboard evidence for the rested pacing change. This SHALL be a narrow exception to timing-related evidence invalidation in scoped classic-realm live evidence and independent classic operation evidence: unchanged profile simulation rules, callback transitions, request construction, realm/authentication contracts, and operation evidence SHALL remain usable despite rested runtime pacing alone.

Rested progression SHALL NOT make an otherwise ineligible character eligible, clear local-only provenance, widen import-path coverage, alter authentication/validators, or fabricate passing evidence. Automatic reports SHALL retain their exact transition snapshots, persistence-before-delivery ordering, serialized delivery, and no-retry behavior.

Documentation and safe managed inspection SHALL distinguish existing operation readiness from unverified rested-timeline acceptance and leaderboard classification; dashboard views SHALL omit the long acceptance notice. Implementing this feature SHALL NOT require or authorize a live conformance experiment. Existing development conformance procedures SHALL retain their original-speed, real-active-time constraints; they SHALL NOT inherit production rested acceleration.

#### Scenario: Reporting a rested eligible character

- **WHEN** an otherwise eligible online browser or desktop character reaches a report-producing transition under rested progression
- **THEN** the runtime attempts the existing profile-compatible report after committing state and accounting, without a new rested-evidence gate

#### Scenario: Retaining existing operation evidence

- **WHEN** existing operation evidence passes its unchanged non-pacing requirements and only production rested timing differs
- **THEN** that timing difference alone does not invalidate the operation's readiness or mark its character local-only

#### Scenario: Preserving an existing gate

- **WHEN** an offline-originated, local-only, unsupported, or otherwise gated character has rested time
- **THEN** its existing reporting restrictions remain in force

#### Scenario: Completing implementation without live verification

- **WHEN** rested progression satisfies its local requirements and focused synthetic checks
- **THEN** implementation can finish without sending live verification requests, issuing new passing evidence, or claiming normal leaderboard classification

#### Scenario: Exercising development conformance tooling

- **WHEN** an existing development conformance runner advances its separately authorized experimental character
- **THEN** it retains the existing unaccelerated active-time contract and does not earn or consume a managed rested bank
