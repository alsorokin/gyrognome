## Context

See `proposal.md` for motivation. The desktop live-conformance runner currently
normalizes response bodies by replacing character identity, authentication
values, passkey text, and submitted profile values with `<redacted>`. Production
guild classification independently replaces only prior/submitted guild values
with `<desktop-guild-designation>`. Their unit tests are internally consistent
but never compare one path with the other.

When primary classification remains indeterminate, guild reconciliation uses a
bounded credential-free public guild lookup against the verified realm page.
That secondary path requires an exact character row, validates the observed
guild against the submitted designation, and never repeats the mutation.

## Goals / Non-Goals

**Goals:**

- Make bundled desktop guild fingerprints reproducible by production.
- Prevent future drift between evidence generation and classification.
- Preserve one-shot mutation semantics and fail closed when neither response
  evidence nor public observation verifies the result.
- Reconcile server-canonical guild capitalization safely.

**Non-Goals:**

- Automatic background synchronization of remote profile state.
- Retrying a guild mutation after an ambiguous result.
- Persisting raw responses, normalized responses, credentials, authenticated
  URLs, or private character files.
- Expanding online eligibility beyond the already evidenced desktop profile,
  realm, endpoint, credential mode, encoding, and import paths.
- Regenerating live evidence unless compatibility with the existing
  normalization contract cannot be established locally.

## Decisions

### Use one shared versioned fingerprint implementation

Move the normalization and SHA-256 logic behind one shared function used by
both the live-conformance runner and production classifier. The function
accepts a response body and an ordered sequence of dynamic values, removes
empty values, replaces each remaining value with the same `<redacted>` marker,
and hashes the resulting UTF-8 bytes.

The shared implementation will preserve the algorithm that produced the
current bundled evidence. This avoids invalidating a successful live experiment
merely to correct production drift.

Alternative: change the evidence generator to production's
`<desktop-guild-designation>` algorithm. This would require new live evidence
because the bundled fingerprints would no longer describe the new algorithm.

### Supply equivalent normalization context in production

Production guild classification will provide the same ordered categories used
by the live runner: character name, account, password, HTTP Basic authorization
value, decimal passkey, prior guild, and submitted guild. Values remain
in-memory only and are never returned, logged, or persisted.

The order is part of the versioned contract because sequential replacement can
behave differently when one dynamic value contains another. Tests will include
overlapping values and a response containing both identity and guild text.

Alternative: normalize only guild values. This is simpler but cannot reproduce
evidence if a response includes another dynamic field and would leave the two
paths semantically different.

### Keep public observation as secondary reconciliation

The response fingerprint remains the primary result. Only an indeterminate
desktop guild response may trigger one credential-free GET against the already
verified realm endpoint, scoped by the exact character name.

The parser identifies the exact leaderboard row and reads only its guild
column. A non-empty observed designation confirms the action only when it
matches the submitted ASCII designation case-insensitively; the observed
canonical spelling is persisted. An absent guild confirms only an explicit
leave. Missing characters, malformed pages, transport failures, and different
guilds remain indeterminate.

Alternative: treat any successful HTTP mutation response as acceptance. This
would incorrectly persist rejected guild names because the legacy endpoint can
return HTTP success for operation-level rejection.

Alternative: synchronize on every dashboard refresh. This would introduce
implicit network activity and conflict with the dashboard's credential-safe,
local-refresh contract.

### Make evidence/production compatibility a direct test boundary

Add shared credential-free vectors that are fingerprinted through the evidence
entry point and then classified through production rules. Tests must not derive
both expected and actual values from separate private helpers. The vectors
cover accepted join/change, invalid rejection, accepted leave, overlapping
dynamic values, and canonical capitalization reconciliation.

## Risks / Trade-offs

- **Existing evidence may depend on undocumented replacement order** ->
  preserve the live runner's exact ordering and add fixed cross-path vectors
  before deleting either old helper.
- **A public page can be delayed after a successful mutation** -> perform only
  one bounded observation and remain indeterminate if the requested state is
  not yet visible; never retry the mutation.
- **HTML markup can change** -> require the exact character row and guild
  column, cap the response size, and fail closed on malformed or missing data.
- **Case-insensitive matching could accept unintended Unicode equivalence** ->
  limit desktop submissions and reconciliation to the existing ASCII contract.
- **The secondary fallback could conceal the primary bug** -> require
  cross-path tests that prove recognized evidence responses complete without a
  public read before testing indeterminate-response reconciliation.
- **Editing main specs before archive would bypass the delta workflow** -> keep
  the reconciliation requirements in this change's delta specs until archive
  synchronizes them.

## Migration Plan

1. Preserve the current local Kenjabob repair; no further leaderboard mutation
   is required.
2. Replace the duplicate fingerprint helpers with the shared versioned
   implementation while keeping the bundled fingerprint values unchanged.
3. Implement the bounded public-profile transport, exact-row parser, canonical
   capitalization handling, and one-read reconciliation defined by the delta
   specifications.
4. Keep the affected main specs at their pre-change requirements so archive
   remains the synchronization point.
5. Run focused cross-path, reporting, transport, parser, and fixture tests,
   followed by default Rust, all-feature Rust, Node, and strict OpenSpec
   validation.
6. Install the validated binary. Roll back by reinstalling the prior revision;
   the reconciled safe guild metadata remains valid.
