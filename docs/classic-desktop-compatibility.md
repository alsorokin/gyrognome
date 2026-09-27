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

The dashboard shows one overall result. A desktop character is eligible only
when every supported operation passes its gate:

- automatic level reporting;
- automatic act reporting;
- manual brag;
- motto changes; and
- guild changes.

Eligibility requires all of the following:

- the `desktop-6.4.4` profile and supported component-stream layout;
- an unadapted import or only `load-spelling-patch`;
- the Spoltog realm;
- the exact saved endpoint `http://progressquest.com/spoltog.php?`, mapped to
  the verified `https://progressquest.com/spoltog.php` destination;
- complete ASCII account/password authentication and a valid passkey;
- ASCII request data;
- current passing evidence for the requested operation; and
- no local-only advancement provenance.

Redirects, other realms or endpoints, passkey-only authentication, unsupported
text encoding, incomplete credentials, substantive adaptations, and local-only
forks are rejected before transport.

## Local-only advancement

Local play is allowed even when automatic level or act reporting is gated. The
first successful advancement in that condition atomically marks the managed
import `LocalOnly`.

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
