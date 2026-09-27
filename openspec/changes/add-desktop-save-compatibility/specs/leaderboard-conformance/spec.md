## ADDED Requirements

### Requirement: Source-derived desktop local conformance

The project SHALL establish local conformance against a separately authored
reference harness derived from the pinned desktop 6.4.4 source rather than the
production implementation. Evidence SHALL record source/build identity,
reference-harness and production implementation identities, relevant numeric
assumptions, synthetic starting state and random state, callback inputs,
ordered transitions, resulting state, and desktop random continuation.
Unsigned request fields, byte encoding, omitted fields, and synthetic validator
results SHALL match at each report point. Evidence without separately approved
official-runtime corroboration SHALL identify exact Delphi compiler/runtime
numeric and random edge equivalence as unverified.

Coverage SHALL include supported 6.2 loading adaptations, fresh prologues,
legacy quest markers, spelling patches, weighted randomness, numeric rounding,
fractional-second task maxima, full-bar callbacks, pause/restart/delay, market
and noncombat progression, level-up, Act II and later acts, and manual/motto/
guild request construction. Browser conformance SHALL continue to pass
unchanged. Fixtures SHALL be generated from synthetic data and SHALL NOT
contain real desktop saves, credentials, authenticated URLs, or raw responses.

#### Scenario: Replaying a desktop checkpoint

- **WHEN** a synthetic desktop checkpoint is replayed
- **THEN** state, ordered transitions, random continuation, and unsigned report
  representation match the pinned source-derived reference observation exactly

#### Scenario: Missing official-runtime corroboration

- **WHEN** the production implementation and source-derived reference agree but
  exact Delphi random or rounding behavior remains uncorroborated
- **THEN** local continuation may use the documented source-derived contract,
  but the limitation remains visible, no exact-runtime claim is made, and the
  evidence cannot enable a classic online operation

#### Scenario: Verifying legacy adaptation

- **WHEN** a 6.2-shaped synthetic save is loaded by the source-derived reference
  harness and the importer
- **THEN** load-time state and subsequent callback transitions match, including
  legacy queues, quest placeholders, and spell collection-order effects, while
  any missing official-runtime corroboration remains disclosed

### Requirement: Scoped classic-realm live evidence

Classic online operations SHALL require separate passing evidence for the
desktop profile, supported import/adaptation path, realm, HTTPS endpoint,
credential mode, encoding, operation, and relevant implementation identity.
Browser/Alpaquil evidence SHALL NOT enable classic reporting. A change to
conformance-relevant behavior SHALL invalidate affected evidence.

A deterministic load normalization MAY share passing live operation evidence
with its canonical import path only when source-derived differential evidence
proves that both inputs converge to identical canonical post-load state and
identical request construction before transport. The recorded
`load-spelling-patch` adaptation MAY qualify through this rule for the
source-derived `Innoculate`/`Inoculate` and
`Tonsilectomy`/`Tonsillectomy` corrections. Progression-affecting, unknown, or
combined adaptations SHALL remain ineligible without separately matching live
evidence.

Live experiments SHALL require separate explicit operator approval of the
realm, disposable account/character scope, operations, and bounds. Only a newly
created disposable character established through the official desktop client
SHALL be used. Client handoff SHALL prevent simultaneous reporters. Live
progression SHALL remain bounded by real active time, with no accelerated
synthetic state or replay of accumulated reports. Evidence SHALL verify
authentication, accepted request semantics, sanitized responses, and normal
rather than cheater leaderboard classification; HTTP success alone is
insufficient. Credentials SHALL remain private and ephemeral to the experiment.
The experiment MAY be staged so immediate operations pass before normal-time
progression begins. An inconclusive exact request intent SHALL NOT be replayed;
a different motto value is a separate intent and MAY be attempted only when
separately approved and within the experiment's explicit attempt bound.
The bounded progression stage SHALL include every distinct level transition
generated before the first act transition. These later level transitions are
separate intents rather than retries. The first act request SHALL be attempted
at most once and ends the required progression trace when verified.

#### Scenario: No approval for live experimentation

- **WHEN** planning or implementation is authorized but a scoped live
  experiment is not
- **THEN** no online character is created or reported by the experiment and
  affected desktop operations remain disabled

#### Scenario: Browser evidence offered for a desktop character

- **WHEN** an operation has passing browser evidence but no matching desktop
  realm/profile evidence
- **THEN** the desktop gate refuses it without contacting the server

#### Scenario: Distinct diagnostic motto after an inconclusive observation

- **WHEN** an approved live experiment has an inconclusive motto observation
  and the operator separately approved another distinct motto within the bound
- **THEN** the runner may submit the distinct intent once, does not replay the
  earlier exact intent, and retains only sanitized intent and response evidence

#### Scenario: An incomplete classic result

- **WHEN** authentication, a required scenario, or normal classification cannot
  be established for the approved disposable character
- **THEN** the result is inconclusive or failed, never passing, and cannot
  enable the affected production operation

#### Scenario: Enabling only evidenced operations

- **WHEN** manual reporting passes for one realm/credential mode but guild
  behavior or a legacy adaptation path remains unverified
- **THEN** only the covered operation and import path become eligible; other
  operations and characters remain gated

#### Scenario: Applying a proven spelling normalization

- **WHEN** a supported desktop import records only `load-spelling-patch` and
  differential conformance proves its post-load canonical state and request
  construction match the evidenced canonical path
- **THEN** the import may use the matching realm, credential-mode, and operation
  evidence while retaining the adaptation in its visible provenance

#### Scenario: Rejecting a substantive or combined adaptation

- **WHEN** an import records a legacy prologue, legacy quest placeholder,
  unknown adaptation, or `load-spelling-patch` combined with another adaptation
  without separately matching live evidence
- **THEN** classic online operations remain ineligible without contacting the
  endpoint
