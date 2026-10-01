# leaderboard-conformance Specification

## Purpose

Establish evidence that Gyrognome's locally advanced characters produce
browser-equivalent leaderboard report histories before general reporting exists.

## Requirements

### Requirement: Browser-equivalent report traces

The system SHALL derive an ordered, credential-free leaderboard report trace
from deterministic character advancement and explicitly requested browser
actions. Each trace entry SHALL identify the browser trigger and the complete
canonical snapshot at the point that the browser emits that report. The trace
SHALL preserve the browser's event order when a single advancement crosses
multiple report-producing transitions.

#### Scenario: Capturing a level-up report snapshot

- **WHEN** an advancement completes a task that reaches a level boundary
- **THEN** the derived trace contains an `l` entry with the canonical snapshot
  after the browser's level-up effects and before later completion effects

#### Scenario: Capturing an act-completion report snapshot

- **WHEN** an advancement completes an act
- **THEN** the derived trace contains an `a` entry at the browser-equivalent
  act-completion point

#### Scenario: Capturing explicit browser actions

- **WHEN** a caller requests a manual brag or changes a motto in the conformance
  input
- **THEN** the derived trace contains respectively a `b` or `m` entry with the
  corresponding browser-equivalent canonical snapshot

### Requirement: Credential-free trace conformance

The system SHALL verify browser-derived report traces using disposable,
synthetic character observations. A trace fixture SHALL record event order,
trigger, canonical snapshot, expected unsigned request fields, normalized
request representation, and validator result using a synthetic passkey.
Committed fixtures and diagnostics SHALL NOT contain player saves, live
passkeys, browser profiles, or complete signed leaderboard URLs.

#### Scenario: Replaying a browser-derived trace

- **WHEN** the system replays a committed report-trace fixture
- **THEN** every derived event's trigger, snapshot, unsigned fields,
  normalized representation, and synthetic validator match the fixture exactly

#### Scenario: Rejecting sensitive trace data

- **WHEN** a report-trace fixture or diagnostic artifact contains a player save,
  live passkey, browser profile, or complete signed request URL
- **THEN** the system rejects it before it can be committed or displayed

### Requirement: Controlled anti-cheat conformance evidence

The project SHALL define a documented, explicitly confirmed Playwright-harness
procedure for external leaderboard conformance experiments. The official
browser client SHALL create the newly created, disposable online character and
its passkey SHALL remain only in the ephemeral browser experiment. The
procedure SHALL compare browser and Gyrognome report histories across initial
load, pause, restart, delayed callbacks, task completion, level-up, act
completion, manual bragging, and motto change. It SHALL additionally exercise
an accepted non-empty guild-designation submission, an accepted empty
guild-designation submission, and a deliberately invalid designation through
the official browser, record their request shape and sanitized accepted or
rejected outcome categories, and restore the disposable character to no guild
before completion.

Guild-response evidence generation and production classification SHALL use one
versioned normalization and fingerprint contract. The contract SHALL replace
the same ordered set of dynamic identity, authentication, passkey, and
prior/submitted guild values with the same placeholder before hashing.
Credential-free cross-path vectors SHALL prove that a fingerprint emitted by
the evidence path is accepted by the production classifier. The procedure
SHALL record only credential-free observations and SHALL require evidence that
the Gyrognome character appears in the normal leaderboard population rather
than the cheater population. Native `newguy` character-generation support is
not required.

#### Scenario: Running an external conformance experiment

- **WHEN** an operator explicitly confirms a disposable-character experiment
- **THEN** the harness creates the character through the official browser,
  sends no reports or guild requests for any existing managed character, and
  records credential-free comparison, guild-outcome, and
  leaderboard-classification evidence for the disposable character

#### Scenario: Observing accepted and rejected guild submissions

- **WHEN** the disposable browser character submits a testable existing guild
  designation, submits a deliberately invalid designation, and then submits an
  empty designation
- **THEN** the evidence records sanitized request field names, operation order,
  safe accepted or rejected categories, and normalized response fingerprints
  produced by the shared versioned contract, without recording a private guild
  designation, passkey, credential, signed URL, or raw response body

#### Scenario: Replaying an evidence fingerprint in production

- **WHEN** a credential-free response vector is fingerprinted by the live
  evidence path and classified by the production guild path with corresponding
  dynamic values
- **THEN** both paths produce the same fingerprint and classification

#### Scenario: Guild response fingerprints are ambiguous

- **WHEN** normalized accepted and rejected guild responses do not produce
  distinct credential-safe fingerprints
- **THEN** guild conformance fails and production guild actions remain disabled

#### Scenario: Guild cleanup cannot be confirmed

- **WHEN** the harness cannot confirm that the disposable character has no
  guild after the empty designation is submitted
- **THEN** guild conformance fails and production guild actions remain disabled

#### Scenario: Evidence is incomplete or classified as cheating

- **WHEN** a required report or guild scenario lacks passing evidence or the
  disposable character is classified in the cheater population
- **THEN** the conformance gate fails and the corresponding general online
  operations remain unsupported

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

### Requirement: Bounded Pemptus manual-report diagnostic

The project SHALL provide a development-only native diagnostic for an explicitly
approved newly created disposable Pemptus character established through the
official desktop client. Execution SHALL require confirmation that the official
client is stopped and that one live manual-brag request is authorized.
Validation-only execution SHALL contact no server.

The diagnostic SHALL accept only a supported desktop-6.4.4 canonical import,
Pemptus realm, exact saved endpoint `http://progressquest.com/pemptus.php?`,
positive passkey, empty account/password credentials, and supported ASCII
request data. It SHALL construct the manual report from the imported state
without progression and send it only to `https://progressquest.com/pemptus.php`,
without an Authorization header, redirects, HTTP downgrades, or mutation
retries. It SHALL preserve source files and SHALL NOT register a managed
character, change motto/guild, or save credentials or signed requests.

#### Scenario: Approved diagnostic execution

- **WHEN** an approved disposable import satisfies all preconditions
- **THEN** at most one native manual-brag request is attempted, and no other
  mutation or simulation advancement occurs

#### Scenario: Invalid or unapproved scope

- **WHEN** confirmation is missing or the realm, endpoint, credentials, encoding,
  or supported import path does not match the diagnostic contract
- **THEN** execution fails with a credential-safe explanation before any network
  activity

#### Scenario: Ambiguous mutation delivery

- **WHEN** the sole mutation attempt times out, disconnects, redirects, or has an
  indeterminate response
- **THEN** its outcome is categorized safely, the mutation is not retried, and a
  later mutation requires new explicit operator authorization

### Requirement: Honest and credential-safe diagnostic observations

The diagnostic SHALL make bounded credential-free public observations of the
exact character before submission and for at most 60 seconds afterward.
Each post-submission observation request and wait SHALL be bounded by the
remaining observation deadline. Responses SHALL have explicit size limits.

Output SHALL distinguish transport delivery, explicit rejection, normal,
cheater, or unknown public classification, and whether observable public state
matches the submitted report. A preexisting unchanged normal row SHALL NOT be
presented as proof that the mutation was accepted. Public-page ambiguity,
malformed data, size-limit failure, or observation timeout SHALL remain
inconclusive rather than being converted into acceptance.

Output SHALL exclude passkeys, credentials, signed URLs, source-save contents,
and raw or normalized response bodies. Diagnostic results SHALL NOT enable
production operations or substitute for full scoped classic-realm live evidence.

#### Scenario: Delivered report with a matching normal row

- **WHEN** a report is delivered and the exact character is observed in the
  normal population with matching report-visible state
- **THEN** delivery, normal classification, and state agreement are reported
  separately, without claiming full Pemptus conformance

#### Scenario: Public state was already identical

- **WHEN** the normal public row matches both the baseline and submitted state
- **THEN** the diagnostic states that mutation acceptance was not independently
  established by the unchanged observation

#### Scenario: Cheater or missing observation

- **WHEN** the exact character is classified as a cheater or cannot be classified
  within the observation bound
- **THEN** execution reports cheater or inconclusive status respectively and
  leaves production reporting eligibility unchanged
