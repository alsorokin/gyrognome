# Classic Desktop Save Compatibility

Gyrognome supports a bounded subset of original Windows Progress Quest saves
without executing the classic client or serialized Delphi components. Desktop
characters use a separate `desktop-6.4.4` continuation profile; they are never
silently converted to browser rules.

## Supported saves

- Content-based import of zlib-compressed Delphi `TPF0` streams observed in
  original 6.2 and 6.4.4 `.pq` saves.
- Same-format `.bak` files are accepted regardless of filename extension.
- Parsing is read-only and bounded. Malformed lengths, unsupported value types,
  conflicting state, unknown layouts, invalid commands or indexes, oversized
  input, and unexplained trailing data are rejected.
- Imported desktop state and private authentication are stored separately. The
  raw desktop save is not retained or modified.

Both observed layouts continue under the pinned 6.4.4 rules. Recognizing a
layout does not prove which executable last wrote a save.

## Load adaptations and spelling normalization

Gyrognome records adaptations instead of hiding them:

- `legacy-prologue62` applies the evidenced 6.2-to-6.4.4 prologue behavior.
- `legacy-quest-placeholder` resolves the carried-forward `fQuest` marker using
  its saved table index.
- `load-spelling-patch` reproduces the classic load-time corrections
  `Innoculate` to `Inoculate` and `Tonsilectomy` to `Tonsillectomy`, except for
  the first spell row because the classic correction loop skips it.

The source save remains unchanged. Simulation and request construction use the
normalized canonical state. Differential fixtures prove that an import with
only `load-spelling-patch` produces the same canonical state and unsigned
level, act, manual brag, motto, and guild requests as an already-canonical
import. Its adaptation remains visible in provenance.

Legacy prologue and quest-placeholder adaptations affect progression semantics
and do not inherit online eligibility from the canonical path. Unknown or
combined adaptations also fail closed.

## Random continuation and history

Classic saves do not persist the original process RNG state, birthday or seed
history, or reliable lifetime task and elapsed counters. Gyrognome therefore:

- initializes a new desktop RNG continuation when the save is registered;
- makes deterministic progress from that new persisted state;
- labels task and elapsed counters as measured since import; and
- does not claim to reproduce the original process's next random choice.

The missing pre-import values remain represented in persisted metadata and JSON
inspection, but the dashboard omits the constant unavailable-history summary.

## Desktop timing and progression

Desktop workers follow callback semantics rather than browser duration
semantics:

- callbacks credit at most 100 milliseconds;
- delayed or missed time is discarded rather than caught up;
- filling a task bar does not complete it until the next actual callback;
- pending full-bar state and desktop RNG state survive restart; and
- level and act reports use their exact transition snapshots.

The dashboard displays the human-readable activity text from the classic state,
while the internal command remains available to simulation.

## Online eligibility

Eligibility is evaluated independently for each supported operation:

- automatic level reporting;
- automatic act reporting;
- manual brag;
- motto changes; and
- guild changes.

The dashboard keeps a concise summary for uniform outcomes. When some
operations are eligible and others are gated, Details shows partial availability
and each operation's safe status/reason. Brag, motto, and guild controls use
their own gates, not an all-five summary.

Mixed-operation lines receive additional Details space when available; compact
terminals show the same safe statuses above activity. No new controls or online
queries are needed to display availability.

Each operation requires all of the following:

- the `desktop-6.4.4` profile and supported component-stream layout;
- an unadapted import or only `load-spelling-patch`;
- an exact supported realm/endpoint/authentication contract and valid passkey;
- ASCII request data;
- current passing evidence for the requested operation; and
- no local-only advancement provenance.

| Realm | Exact saved endpoint | Fixed HTTPS destination | Authentication |
|---|---|---|---|
| Spoltog | `http://progressquest.com/spoltog.php?` | `https://progressquest.com/spoltog.php` | Complete ASCII account/password; Basic authorization |
| Pemptus | `http://progressquest.com/pemptus.php?` | `https://progressquest.com/pemptus.php` | Both account/password empty; no Authorization header |

Recognizing a contract does not enable reporting. Bundled passing evidence
currently covers all five Spoltog operations and only Pemptus manual brag and
motto set/clear. The two Pemptus records were independently observed in the
approved immediate stage and promoted after final no-guild/private cleanup.
They have separate integrity pins and are valid through October 1, 2027.
Pemptus automatic level/act and guild actions remain gated; the operator-accepted
join has no original response fingerprint and cannot provide guild evidence.
This is partial support, not imported Pemptus parity. Operation-scoped evidence
does not require unrelated operation coverage or guild fingerprints
for non-guild actions; the original Spoltog v2 envelope retains its original
integrity and scope.

Redirects, other realms or endpoint aliases, mismatched credential modes,
unsupported text encoding, incomplete credentials, substantive adaptations,
and local-only forks are rejected before transport.

## Local-only advancement

Local play is allowed even when automatic level or act reporting is gated. The
first successful advancement in that condition atomically marks the managed
import `LocalOnly`.

Missing only manual, motto, or guild evidence does not cause this fork when
both automatic operations are eligible. Conversely, manual/profile availability
does not make local advancement safe for later online use if either automatic
operation is gated.

With current bundled Pemptus coverage, manual brag and motto actions are available
on an untouched supported import, but its first local advancement permanently
closes every online operation. Automatic evidence installed later cannot reconnect
that managed timeline.

This is permanent for that record because classic reports describe exact level
and act transitions rather than a general current-state synchronization.
Gyrognome does not queue or replay reports that were missed while gated.
Installing evidence later, replacing credentials, reenrolling, or clearing a
flag cannot safely reconnect that locally advanced timeline.

The dashboard hides the normal `Unadvanced` value. A local-only record instead
shows:

```text
Online progression: local-only; fresh official-client import required
```

The record remains usable locally. Future online use requires a fresh save
imported from the official client after all relevant gates pass. Missing
guild-only evidence does not by itself create a local-only fork; progression
gating is based on automatic level and act reporting.

## Privacy and encoding

- Desktop state, authentication, motto, and guild text initially support ASCII.
- Browser Unicode behavior is unchanged.
- Passkeys, account names, passwords, raw or authenticated endpoints, raw save
  properties, signed URLs, and raw responses are excluded from normal output,
  logs, fixtures, and dashboard presentation.
- Registering, inspecting, listing, refreshing, starting, and stopping do not
  independently send online requests.
- Delivery is one-shot. Failed automatic or explicit requests are not retried.

## Conformance evidence

Local continuation is checked against a separately authored reference harness
derived from the pinned desktop 6.4.4 source. Exact Delphi compiler/runtime
numeric edge equivalence remains unverified; this limitation does not prevent
local play and does not independently authorize online delivery.

Classic online operations require separate disposable live evidence scoped to
profile, import path, realm, endpoint, credential mode, encoding, operation,
and implementation identity. Browser or Alpaquil evidence cannot enable
desktop operations.

An initial Spoltog experiment on September 25, 2026 was inconclusive and
enabled nothing. A fresh experiment completed on September 27, 2026 and
established normal-classification evidence for:

- automatic level reports generated before the first act report;
- the first act report;
- manual brag;
- motto set and clear; and
- accepted, rejected, and empty guild operations.

Requests were one-shot, the official client and Gyrognome were never active as
simultaneous reporters, and private save/checkpoint artifacts were removed
afterward. Bundled production evidence enables only that exact Spoltog scope,
plus the source-proven equivalent `load-spelling-patch` path.

The development-only dry-run guard remains available:

```sh
node scripts/desktop-live-conformance.mjs \
  --confirm-disposable \
  --realm "Approved realm" \
  --account-scope new-disposable-account \
  --character-scope new-official-client-character \
  --operations manual-brag,motto,guild \
  --max-active-seconds 300 \
  --confirm-client-handoff
```

The guard validates approval inputs and produces an intercepted sanitized
manifest; it does not launch the official client or submit a live request.
Live execution additionally requires the feature-gated
`desktop-live-conformance` Rust binary and explicit reviewed approval.

## Scoped Pemptus conformance stages

The native feature-gated runner supports separately approved immediate and
progression stages. It does not create desktop characters, start the official
client, or reuse the earlier probe's exhausted authorization. The JavaScript
dry-run manifest does not authorize a live Pemptus stage.

Before each stage, approve a newly created disposable official-client identity,
the official control operations, native operations and attempt bound,
observation deadlines, exclusive stopped-client handoff, and exact cleanup
paths. Establish the approved control motto through the official client, stop
it, and keep every other reporter stopped. Use an unadapted, level-1,
game-style-3 save with no guild.

An explicitly approved exception allows an older disposable Pemptus character
for the immediate stage only. It must still pass the level-1, initial
plot/quest, unadapted-import, and authentication checks. Use private copies;
the original exploratory save and backup remain untouched. No official-client
mutation is authorized by this exception, and every reporter must remain
stopped.

With `--allow-existing-disposable`, the existing saved/public control motto may
be empty. The runner first requires exact normal public-row and current-guild
agreement with the copied save; a stale or unidentifiable baseline stops before
mutation. Manual and motto-operation markers must still be distinct non-empty
ASCII strings and differ from the control baseline.

If the baseline already has a guild, separately approve `--initial-guild-leave`.
This preparation is recorded before delivery and must be conclusively observed.
It does not replace final guild-leave proof or grant another operation coverage.
Full immediate coverage requires eight native attempts including preparation.
An uncertain preparation cannot be replayed, even after restart.

Pemptus public observation accepts exactly one omitted empty trailing guild
cell only in a table explicitly declaring the complete leaderboard header,
including Guild (with its optional displayed Ctrl-G shortcut). The unique
exact-name row must contain all preceding cells. Missing headers, spans,
duplicate rows, extra/malformed cells, and unlinked non-empty guilds remain
unknown. Production guild reconciliation and the scoped development runner use
the same reader; Spoltog retains its historical strict interpretation.
Guild names use the same ASCII-case-insensitive reconciliation in production
and development, retaining canonical public spelling; character names still
match exactly.

The only recovery exception requires fresh explicit approval of a **60-second
read-only** window for the sole recorded preparatory leave. Keep all original
scope arguments, copied source bytes, and the eight-attempt cap unchanged, and
add both:

```sh
--preparation-reconciliation-seconds 60 --confirm-preparation-reconciliation
```

These flags do not authorize recovery by themselves: obtain approval before
execution. Recovery requires the existing-disposable immediate stage, all
three immediate operations, exactly one matching preparatory intent, zero
observations, and no callback progress. It reads the normal exact-name row
within the fresh bound and requires saved report-field agreement and confirmed
no guild. A changed source/scope, unknown primary intent, mismatch, unknown
classification, or expired window blocks continuation without mutation.

Successful recovery retains the original intent/count, records authorization
and actual observation timing separately in the private checkpoint, and does
not fabricate the missing response fingerprint or validate the expired
observation. Only the seven unattempted primary cases then run under the
original eight-attempt cap, each with fresh bounded acceptance observations.
Recovery is not operation evidence and cannot replace final guild-leave proof
or cleanup. It never applies to progression; that stage still requires its
separately approved newly created disposable identity and real active time.

### Operator-accepted join continuation

A separate exception applies only to the approved 2026-10-01 immediate
checkpoint: five total attempts, three conclusive manual/motto observations,
completed preparation recovery, no callback progress, and the sole pending
`BEERGuild` join. The operator confirmed no competing join request and accepted
the linked public `BEERguild` membership. The runner must not repeat the
preparation, manual report, motto set/clear, or join.

Keep the original source and scope arguments, including the eight-attempt cap,
`--guild-designation BEERGuild`, `--guild-change-designation QoD`, and 60-second
primary observation bound. After explicit approval, add:

```sh
--operator-join-reconciliation-seconds 60 \
--confirm-operator-accepted-join --confirm-no-competing-join
```

Use `--validate-only` first to verify the exact source, scope, prefix intents,
and checkpoint without network activity. The live continuation opens a fresh
60-second read-only window to verify normal report-field agreement and the
approved linked guild. It records operator authorization and actual
reconciliation timing separately, preserving the original ledger and missing
response fingerprint. Only guild change to `QoD`, invalid-designation
rejection, and final leave can follow, with three fresh bounded observations
under the original eight-attempt total cap. A mismatch, other uncertain primary
case, changed scope/source, abnormal classification, or expired window stops
continuation without replay. Reporters must remain stopped.

Operator acceptance is lifecycle completion, not a timely automated join
observation or guild production evidence. After final no-guild state and
approved private-only cleanup, the runner emits only independently complete
manual/motto records and explicitly reports guild exclusion because its
original join response fingerprint is absent. Production guild gates remain
closed; originals remain untouched. This does not authorize progression or
general recovery of a primary operation.

Place only that disposable `.pq`, its matching optional `.bak`, and
runner-owned state in a private directory outside all Git checkouts. The
directory must have mode `0700` and its files mode `0600`. Symlinked inputs and
unrelated backup stems are refused. No real save, credential, or signed request
belongs in a checkout or committed fixture.

For the full immediate stage, approve two distinct valid existing ASCII guild
designations using their exact public spelling. Seven native mutation attempts
cover manual brag, motto set/clear, guild join/change, rejected invalid
designation, and leave. A distinct approved manual motto marker makes report
consumption observable without inventing progress or sending an intervening
motto request. It is not evidence for the separate motto operation.

After approval and handoff, first validate the scope without network activity:

```sh
cargo run --features desktop-live-conformance --bin desktop-live-conformance -- \
  --realm Pemptus --stage immediate \
  --operations manual-brag,motto,guild --max-mutation-attempts 7 \
  --experiment-dir /path/to/private-pemptus-immediate \
  --evidence /path/to/pemptus-immediate-evidence.json \
  --control-motto "Approved official control" \
  --manual-motto "Approved manual marker" --motto "Approved motto marker" \
  --guild-designation "Approved guild A" \
  --guild-change-designation "Approved guild B" \
  --classification-poll-seconds 60 \
  --confirm-disposable --confirm-client-stopped \
  --confirm-live-submission --confirm-cleanup --validate-only
```

Remove `--validate-only` only for the separately authorized live execution.
Approved subsets such as `--operations manual-brag,motto` can complete without
guild coverage; their attempt bound must cover every requested subcase.
The markers must be distinct, non-empty ASCII values.

For the approved existing-character exception, add
`--allow-existing-disposable`; if initially guilded, also add
`--initial-guild-leave` and replace the seven-attempt bound with
`--max-mutation-attempts 8`. For an approved empty baseline, replace the control
argument with `--control-motto ""`. These options are refused for progression
and legacy stages, and cannot be inferred from a save file.

Progression requires a separate disposable identity, fresh approval, both
automatic operations, an explicit native mutation cap, and six to eight hours
of active callback time. For example, replace the stage-specific arguments with:

```sh
--realm Pemptus --stage progression \
--operations automatic-level,automatic-act --max-mutation-attempts 32 \
--max-active-seconds 28800
```

Use distinct approved private/evidence paths and the same required confirmation
and control-motto flags. Progression uses actual callbacks capped at 100 ms,
without acceleration or stopped-time catch-up. Observation requests have
bounded timeouts and sleeps clipped to the remaining deadline.

The runner records intent before mutation. An uncertain request or interrupted
callback cannot be replayed on restart. Inconclusive stages retain private
state and emit a non-enabling safe diagnostic; obtain new guidance/approval
before further mutation or cleanup. HTTP 200, an empty body, and an unchanged
agreeing row are not acceptance evidence.

Successful stages verify normal classification and no-guild cleanup, remove
only the approved disposable save/backup and native checkpoint, and emit a JSON
collection of individually validated v3 operation records. Their dates refer
to actual operation observations, not a later restart. Only reviewed records
with pinned integrity and matching scope can be bundled into production.
Partial coverage enables only proven operations, and complete Pemptus support
is not claimed until all five operations and required subcases pass.

Spoltog's existing invocation remains the default legacy mode. Stage-specific
operation/attempt options cannot silently narrow that legacy experiment: use
an explicit scoped stage instead. Native desktop enrollment remains unsupported.

## Pemptus manual-report diagnostic

The development-only `pemptus-acceptance-probe` uses the native desktop importer
and revision-8 manual-report constructor for a canonical, level-1 Pemptus
passkey-only save. It does not enable Pemptus production reporting or replace
the full live conformance experiment.

Use only a newly created disposable character established through the official
desktop client. Stop that client before handoff and keep it stopped throughout
the diagnostic. Validate the approved save without contacting a server:

```sh
cargo run --features desktop-live-conformance --bin pemptus-acceptance-probe -- \
  --save /path/to/disposable.pq \
  --confirm-disposable --confirm-client-stopped --validate-only
```

After explicit approval for one live manual report:

```sh
cargo run --features desktop-live-conformance --bin pemptus-acceptance-probe -- \
  --save /path/to/disposable.pq \
  --confirm-disposable --confirm-client-stopped --confirm-live-submission
```

The probe first reads a bounded public baseline, attempts at most one manual
brag to the fixed Pemptus HTTPS endpoint without Basic authentication, then
observes the exact public character row for at most 60 seconds. Each request
has a timeout and response-size limit; redirects are refused. It never
advances simulation, edits motto/guild, registers a managed character, retries
the mutation, changes source saves, or writes private experiment artifacts.

Output separates delivery, response category, normal/cheater/unknown
classification, report-visible state agreement, and observable state change.
A matching row that was already present does not independently establish that
the report was accepted. Even an observed change is corroboration, not a
validated server response contract or proof of full conformance. Production
eligibility remains unchanged.

A failed or ambiguous mutation attempt may already have reached the server.
Do not rerun the probe without renewed explicit operator authorization; each
invocation is a separate live mutation scope. Source saves, credentials,
signed URLs, and raw responses must stay out of fixtures and committed files.
