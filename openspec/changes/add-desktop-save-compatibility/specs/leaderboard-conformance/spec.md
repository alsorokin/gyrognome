## ADDED Requirements

### Requirement: Independent desktop local conformance

The project SHALL establish network-blocked conformance against the pinned
desktop 6.4.4 client behavior independently of the production implementation.
Evidence SHALL record oracle source/build identity, relevant runtime numeric
behavior, synthetic starting state and random state, callback inputs,
ordered transitions, resulting state, and desktop random continuation.
Unsigned request fields, byte encoding, omitted fields, and synthetic validator
results SHALL match at each report point. A second transcription of unverified
port assumptions SHALL NOT count as independent evidence.

Coverage SHALL include supported 6.2 loading adaptations, fresh prologues,
legacy quest markers, spelling patches, weighted randomness, numeric rounding,
fractional-second task maxima, full-bar callbacks, pause/restart/delay, market
and noncombat progression, level-up, Act II and later acts, and manual/motto/
guild request construction. Browser conformance SHALL continue to pass
unchanged. Fixtures SHALL be generated from synthetic data and SHALL NOT
contain real desktop saves, credentials, authenticated URLs, or raw responses.

#### Scenario: Replaying a desktop checkpoint

- **WHEN** a synthetic desktop checkpoint is replayed
- **THEN** state, ordered transitions, random continuation, and unsigned
  report representation match the independent observation exactly

#### Scenario: Missing independent numeric evidence

- **WHEN** the production implementation and a second port agree but Delphi
  random or rounding behavior remains unverified
- **THEN** the affected conformance milestone remains incomplete

#### Scenario: Verifying legacy adaptation

- **WHEN** a 6.2-shaped synthetic save is loaded by the desktop oracle and the
  importer
- **THEN** load-time state and subsequent callback transitions match, including
  legacy queues, quest placeholders, and spell collection-order effects

### Requirement: Scoped classic-realm live evidence

Classic online operations SHALL require separate passing evidence for the
desktop profile, supported import/adaptation path, realm, HTTPS endpoint,
credential mode, encoding, operation, and relevant implementation identity.
Browser/Alpaquil evidence SHALL NOT enable classic reporting. A change to
conformance-relevant behavior SHALL invalidate affected evidence.

Live experiments SHALL require separate explicit operator approval of the
realm, disposable account/character scope, operations, and bounds. Only a newly
created disposable character established through the official desktop client
SHALL be used. Client handoff SHALL prevent simultaneous reporters. Live
progression SHALL remain bounded by real active time, with no accelerated
synthetic state or replay of accumulated reports. Evidence SHALL verify
authentication, accepted request semantics, sanitized responses, and normal
rather than cheater leaderboard classification; HTTP success alone is
insufficient. Credentials SHALL remain private and ephemeral to the experiment.

#### Scenario: No approval for live experimentation

- **WHEN** planning or implementation is authorized but a scoped live
  experiment is not
- **THEN** no online character is created or reported by the experiment and
  affected desktop operations remain disabled

#### Scenario: Browser evidence offered for a desktop character

- **WHEN** an operation has passing browser evidence but no matching desktop
  realm/profile evidence
- **THEN** the desktop gate refuses it without contacting the server

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
