## Context

See `proposal.md` for motivation and the capability deltas for acceptance
criteria. The current import path reads Base64 JSON text, canonical state
embeds browser Alea assumptions, and the database retains original browser JSON
as its credential source. Browser simulation and reporting already have
synthetic checkpoints, a developer-only conformance bridge, and a disposable
Playwright experiment. They are regression contracts, not a desktop oracle.

Read-only inspection established that the supplied desktop saves use zlib
around concatenated Delphi `TPF0` components. The observed layout has 37
components; 1,312 game-list rows were checked across nine saves. This validates
the observed list framing, not arbitrary Delphi serialization.

Desktop source provenance is the official Bitbucket repository `grumdrig/pq`,
tag `v6.4.4`, commit `95f5d66f97a3697a7446fba951ab04156c752274`.
The supplied 6.4.4 executable has SHA-256
`fca7602ed8212bcdafa96d5f35c619142c54396ac54e659c156b967edaeba17c`;
its spell and monster tables match the tagged resources. Its stale Windows
version resource is not the application version: the operator confirmed 6.4.4
on its welcome screen. The executable hash was reverified without launching it
on September 25, 2026. The earlier `v6.2` source tag is useful comparative
evidence but is not an independently established exact binary match.

No executable was launched and no server experiment was performed during
planning. Save-format observations and source analysis do not prove Delphi
numeric execution or classic server acceptance.

## Goals / Non-Goals

**Goals**

- Isolate desktop behavior behind explicit typed profile dispatch, without
  refactoring browser algorithms merely to make the two look alike.
- Separate untrusted decoding, semantic adaptation, deterministic execution,
  persisted provenance, and authorized delivery.
- Make unsupported inputs and incomplete evidence visible, with no silent
  browser fallback or success-shaped conformance result.
- Support local use independently of live classic-realm approval.

**Non-Goals**

- General-purpose Delphi deserialization, desktop export, arbitrary legacy
  code-page support, or round-tripping raw desktop UI properties.
- A native implementation of classic account/character creation or reenrollment.
- Reconstructing unsaved original randomness or reproducing a saved character's
  exact future trajectory in the original process.
- Retrofitting unrelated browser new-guy, administration, or dashboard behavior.

## Decisions

### 1. Make source-derived desktop evidence the first implementation milestone

Use a developer-only reference harness derived from the pinned desktop 6.4.4
source, separately authored from the Rust production implementation. Drive
synthetic states with known random state and controlled callback inputs.
Capture semantic checkpoints and unsigned report observations, then compare
the production implementation against those fixtures. Separately approved,
network-blocked execution of the verified official executable may corroborate
selected load, timer, numeric, and report transitions, but it is not a
prerequisite for local implementation. Keep proprietary compiler components
and executables outside the repository.

The initial reference work must cover source-observed bounded `Random`,
`Random64`, overflow/range behavior, weighted-stat sampling, integer division,
floating precision/`Round` at XP thresholds, and byte encoding. A second
Rust/JavaScript transcription does not prove exact Delphi compiler/runtime
equivalence, so fixture metadata and user-facing documentation must identify
unverified numeric/random edge behavior. That limitation does not block local
continuation, but it cannot enable classic online operations or support an
unqualified exact-runtime claim.

Pin source/build and observation fingerprints in sanitized JSON metadata.
Record actual and expected callback sequences, not just aggregate durations.
Preserve existing browser fixture shapes and use a separate desktop fixture
schema. Extend the existing bridge only at its explicit developer boundary;
normal builds must not expose fixture injection or synthetic reporting.

Alternative rejected: treating source-derived agreement as proof of exact
Delphi runtime equivalence or classic leaderboard compatibility. Local support
may use the source-derived contract, but online enablement remains independently
gated.

### 2. Decode bytes into a narrow validated desktop import model

Change the save entry point to bounded byte input with explicit browser versus
desktop content detection. Keep the existing browser text decoder/export path
intact. A malformed recognized container must fail within its format, not be
retried as an unrelated format to hide an error.

Add a direct Rust zlib dependency and a structural Delphi reader. Do not scan
for `TPF0` markers or property names inside payloads. Parse supported value tags,
component boundaries, list framing, and terminators with checked arithmetic.
Initial parser limits are 8 MiB compressed input, 64 MiB inflated output, depth
32, 256 components, 1 MiB per length-delimited value, and 100,000 aggregate
list rows. Treat these as declared support limits rather than properties of
all desktop saves; tests must cover each exact boundary and rejection above it.
Validate component names/classes against the supported layout. Safely consume
known framing for irrelevant UI properties, but reject unknown value types and
conflicting state-bearing components.

The observed `Items.Data` contains a total-length word, row count, five signed
32-bit row-header words, one-byte-length caption/subitem strings, and a trailing
two-byte entry per subitem. Validate the total size, header subitem count,
trailer size, and exact exhaustion together. Do not use row counts to allocate
unchecked memory.

Use a mapping table with evidenced property defaults. Important mappings:

| Data | Desktop source |
|---|---|
| Ordered character collections | Traits, Stats, Equips, Inventory, Spells, Plots, Quests |
| Current task / quest marker and index | fTask, fQuest caption and tag |
| Pending commands | fQueue string list |
| Activity display | Kill.SimpleText |
| Numeric task/XP/quest/plot progress | Bar Position and Max |
| Prized equipment / game style | Equips.Tag / InventoryLabelAlsoGameStyle.Tag |
| Passkey | Traits.Tag plus Traits.Hint |
| Motto / guild | Stats.Hint / Label1.Hint |
| Realm / endpoint | Spells.Hint / Equips.Hint |
| Account / password | Inventory.Hint / Plots.Hint |

Empty lists and zero properties can be omitted legitimately. Visual progress
hints are not authoritative. `Timer1.Tag` is a process timing baseline, not
elapsed history. The supported state-bearing text subset is ASCII; reject
non-ASCII bytes with a categorized encoding error. Never print a failing
property's value. Future code-page support requires a separate evidenced
contract rather than heuristic decoding.

Generate compressed binary fixtures in memory from reviewed synthetic inputs.
Keep committed fixtures textual JSON, consistent with `tests/fixture_safety.rs`.
Neither real saves nor purportedly anonymized copies enter tests.

Alternative rejected: wrapping parsed desktop values in a fabricated browser
JSON document. That confuses absent browser history with imported facts and
lets Alea/protocol defaults leak into desktop continuation.

### 3. Persist typed compatibility and private authentication separately

Introduce a closed profile identity with browser and desktop-6.4.4 variants.
Keep shared display/game fields where meanings agree; represent incompatible
random state and profile-specific continuation as variants. Select rules,
simulation entry points, protocol construction, and evidence through this
identity, not through realm names or runtime guesses.

Separate source provenance from continuation identity. A save has no reliable
universal version stamp: record recognized layout/adaptation and any
operator-declared origin without asserting a writer version. Fresh and
6.2-carried-forward fields must be interpreted by evidenced 6.4.4 load
behavior. In particular, cover legacy prologue commands, `fQuest`, and the
load-time spell spelling loop that skips the first row. Preserve desktop table
order and distinct spell identities.

Represent missing lifetime history explicitly. Initialize task/elapsed counters
as local measurements labeled since import, not historical totals. Generate
new desktop random state only for registration/continuation, with deterministic
injection restricted to tests. Pure inspection does not manufacture a claim
about the original next random result.

Version the canonical representation and migrate the current schema-2 store
transactionally. Existing records receive the browser profile with unchanged
Alea fields, original JSON, identifiers, and report behavior. Reject unknown
versions; do not default an unknown variant to browser. Keep browser export
semantics and reject unsupported desktop export explicitly.

Retain desktop passkey, account/password, and original endpoint information in
private storage, distinct from serde-visible safe projections. Do not retain
raw binary saves. Preserve existing user-private database/directory permissions,
including SQLite sidecars and any migration backup. Character removal must
delete every new private row atomically. This uses the existing local private
store trust boundary, not a claim of encryption at rest.

### 4. Keep desktop callback semantics separate from browser duration semantics

Reuse common pure helpers only when exact behavior agrees. Bundle a pinned
desktop table set; preserve spelling and order rather than aliasing browser
entries. Keep desktop integer randomness and numeric operations separate from
`rng.rs` browser Alea primitives.

The desktop pure entry point consumes callback inputs. A callback either
dispatches an already-full task or advances its bar by clamped elapsed time,
never both. A 6,000 ms task takes 60 regular 100 ms advancement callbacks and a
61st callback to dispatch completion. Partial milliseconds and completed-task
credit have different units: applicable credit is `TaskBar.Max div 1000`.
Non-loading market and other noncombat tasks can advance plot progress.
Act II awards an item only; later acts also award equipment.

Capture `l` immediately after level-up effects and before later completion
effects. Capture `a` within CompleteAct, before dequeue removes the plot command
and selects the next loading task. Do not reuse the final canonical state as a
substitute for an intermediate report snapshot. Desktop cinematic/loading
durations and legacy queues come from pinned source-derived evidence and
fixtures, not browser convenience.

Schedule desktop workers at their evidenced 100 ms callback cadence rather than
browser boundary-aligned catch-up. Use actual monotonic elapsed time capped at
100 ms. Do not invent callbacks after suspension, batching, or network delay.
Reset the baseline after callback processing; restart establishes a new baseline.
Persist full-bar pending completion and random state so a crash cannot turn a
full bar into an immediate browser-style completion or duplicate rewards.

Commit complete callback state before delivering its ordered in-memory report
snapshots. A crash after commit may lose a best-effort report; it must not replay
the callback or queue the report on restart. Existing exclusive runtime and
short-lived online-action locks remain authoritative. Explicit actions cannot
overwrite concurrent simulation state; worker reports use transition snapshots
plus profile values coherent with their serialized action order.

Alternative rejected: globally fixing browser timing/rewards. Those differences
are required browser behavior and must continue passing browser checkpoints.

### 5. Centralize profile-aware delivery eligibility and fork provenance

Replace repeated browser-only eligibility decisions with one shared typed
decision consumed by worker, CLI, dashboard, motto, and guild actions. It
returns safe reason categories, never credentials. Keep existing browser
gates unchanged and add desktop gates for profile/build identity, adaptation,
realm, endpoint, credential mode, encoding, operation, and advancement provenance.

Classify adaptations by their effect on the post-load state used by reporting,
not merely by whether an importer recorded an adaptation. The source-derived
6.4.4 spelling loop deterministically changes `Innoculate` to `Inoculate` and
`Tonsilectomy` to `Tonsillectomy` after loading. Treat that
`load-spelling-patch` marker as equivalent to an already-canonical spelling
only when differential tests prove identical canonical state and desktop
request construction after load. The marker remains visible in provenance.
Do not extend this equivalence to legacy prologue or quest-placeholder
adaptations, which affect progression state or transitions and remain
separately evidence-gated. Unknown or combined adaptations fail closed.

Per the operator's explicit decision, local advancement is allowed while
classic progress reporting is gated. Atomically mark the first such advancement
as a local-only fork. The mark is monotonic for that managed import. Registration
or inspection without advancement does not set it. Missing guild-only evidence
does not by itself fork progress when all required progression evidence passes.
An ordinary one-shot network failure under passing evidence is not the same
as gated progression and keeps the existing best-effort policy.
One-shot identity is the exact persisted request intent, not the broad action
kind. The live conformance harness may permit a separately approved, bounded
second motto value after an inconclusive first observation, but it must never
resend either exact request intent. This diagnostic exception does not add
automatic retries to production motto handling.

The live handoff's "fresh" requirement means a newly created level-one
official-client character, not a byte-for-byte untouched initial save. A
bounded amount of official-client plot progress needed to establish and verify
the approved control state is acceptable; higher level, extended plot history,
or multiple quest-history entries remain outside the disposable experiment.

During the bounded normal-time progression stage, every distinct naturally
generated level report before the first act report is part of the approved
trace. A later level report is not a retry of an earlier level request because
its transition snapshot and intent digest differ. The first act report ends the
required progression trace; neither an exact level intent nor the act intent may
be replayed.

When evidence becomes available, a fork cannot regain eligibility by toggling
a flag, reenrolling, replacing credentials, or replaying missed events. Require
a fresh import from the official client for future online use; leave the fork
available for local play. A binary save cannot cryptographically prove its own
origin, so document the required official-client provenance and one-client
handoff rather than advertising tamper-proof attestation.

Use independent operation gates, not one global "desktop works" boolean.
Spoltog is a motivating realm, not implicit authorization or an automatically
trusted endpoint. Retain imported endpoint metadata privately, and map only
recognized official destinations to explicitly verified HTTPS endpoints.
Reject redirects that change destination or downgrade transport. Never send
credentials to an arbitrary save-supplied URL.

Desktop source applies account authentication on ongoing requests, in addition
to the passkey-derived validator. Preserve both; choose the HTTPS authentication
representation only after the scoped experiment proves it. Unsupported modes
remain gated rather than dropping account data or enabling HTTP. Production
request construction has no transport side effects.

Keep revision 8 and desktop field order/encoding, including absence of `z`
when spells are empty. Motto/guild input remains Unicode for browser characters;
the initial desktop path rejects non-ASCII before mutation. Response
fingerprints and bounded normalization are scoped to desktop realm/operation;
browser fingerprints cannot authorize a classic guild change. HTTP 200 means
delivery, never proof of normal leaderboard classification.

### 6. Separate local completion from approval-dependent online enablement

The local milestone includes decoder, profile migration, source-derived desktop
continuation, safe UI, and a closed-by-default delivery gate. It can be useful
without any production request. README/help must call it local desktop support,
disclose that exact Delphi numeric/random edge equivalence is unverified, and
state that classic operations remain unavailable.

The live milestone requires new explicit approval of realm, disposable
account/character, operations, real-time bounds, and cleanup. The official
desktop client creates the disposable identity; stop it before native handoff.
Do not use existing player saves or run both reporters simultaneously. Match
real-time progression and observe authentication, operation outcomes, and
normal-versus-cheater population with bounded polling. Unknown classification,
ambiguous guild fingerprints, incomplete cleanup, or an adaptation without
either matching live evidence or an established canonical/protocol equivalence
fails the relevant gate.

Store only sanitized evidence: source/build identity, field names and unsigned
synthetic representations, outcome categories, normalized fingerprints, and
classification results. Live credentials and authenticated URLs never enter
fixtures, logs, the repository, or ordinary test artifacts. If approval is
withheld, leave the live tasks blocked and production gates closed; do not mark
them complete because a dry run succeeds.

## Risks / Trade-offs

- Unknown Delphi numeric/compiler behavior -> derive local behavior from the
  pinned source and synthetic differential fixtures, disclose unverified edge
  equivalence, and never use source-derived agreement to enable classic online
  operations.
- Ambiguous legacy load state -> support only evidenced adaptations, including
  default omissions and migration quirks; reject unsupported states safely.
- No embedded writer version -> separate provenance and target profile;
  field shape is not proof of the last executable used.
- ASCII-only initial support excludes some real names/credentials -> clearly
  document the limit and reject without loss; do not guess an ANSI code page.
- Frequent desktop checkpoint writes -> verify the current SQLite persistence
  path sustains real-time callbacks; never hide latency with simulated catch-up.
- Old binary cannot understand new canonical variants -> require backup-based
  rollback, never lossy down-conversion of desktop records.
- Server rules/authentication are not public evidence -> realm/operation gates
  stay closed until a scoped live result passes; local correctness alone does
  not imply leaderboard acceptance.
- Gated progress cannot later go online -> surface the permanent local-only
  consequence before starting an ineligible online-originated character and
  keep a safe persistent status visible.

## Migration Plan

1. Establish source-derived synthetic desktop and browser regression baselines.
2. Add decoder, typed profile state, transactional browser-store migration,
   and safe projections with all desktop online gates closed.
3. Enable supported local desktop continuation only after local conformance;
   wire runtime/CLI/dashboard and local-only provenance as one release unit.
4. Implement revision-8 request/transport adapters behind scoped gates. Run
   network-blocked negative-path tests before any live experiment.
5. Obtain separate live approval; enable only operations/import paths backed by
   matching sanitized passing evidence or a narrowly established
   canonical/protocol-equivalent load normalization. If approval or evidence is
   missing, retain the local milestone and report the online milestone as
   blocked.
6. For rollback, stop workers and use a user-private pre-migration backup for
   the older binary. Warn that newer progress is absent from that backup.
   Never automatically overwrite current state or coerce desktop characters
   into browser records.

## Open Questions

- Which separately approved offline path, if any, should be used to corroborate
  selected source-derived observations against the verified official
  executable? Missing corroboration does not block local support, but remains
  visible as an evidence limitation.
- Which classic realms and credential modes can demonstrate an acceptable
  HTTPS authentication contract? None is enabled until verified individually.
- Which response fingerprints and bounded classification observations are
  stable for an approved realm? Unknown outcomes remain safely indeterminate.
