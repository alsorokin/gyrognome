# Design

## Context

See `proposal.md` for motivation and the delta specs for observable behavior.
The existing protocol module already constructs browser-compatible
motto-change reports and guild requests. Production delivery currently exposes
only manual brag and worker-generated level/act reports, all profile values are
effectively empty, and foreground reporting acquires the worker's lifetime
ownership lock.

The official browser assigns `game.motto` and `game.guild` as top-level values,
saves the complete game object, sends motto changes as a `t=m` progress report,
and sends guild changes through `cmd=guild`. The current Rust parser treats
these optional fields as unrecognized data. The worker also passes an empty
motto into simulation, so later automatic reports cannot reproduce a user's
selected motto.

The managed store keeps canonical state and the credential-bearing original
document separately. A worker replaces the complete canonical-state JSON on
each tick while holding a lifetime advisory lock. Dashboard state is currently
read-only except for delegated lifecycle and manual-brag operations.

## Goals / Non-Goals

**Goals:**

- Preserve motto and guild metadata without creating lost-update races with
  active simulation persistence.
- Permit explicit profile actions while the worker retains simulation
  ownership.
- Give every online request for one character a deterministic order and a
  coherent state/profile snapshot.
- Keep guild response interpretation evidence-based and credential-safe.
- Reuse existing terminal interaction, endpoint validation, and safe outcome
  conventions.
- Make the requested dashboard layout changes without altering compact-mode
  behavior or deterministic simulation.

**Non-Goals:**

- Creating, administering, searching, or enumerating guilds.
- Proving server-side leaderboard classification from an HTTP success.
- Retrying or queueing failed profile or report requests.
- Synchronizing profile changes made later by another browser installation.
- Adding profile fields to deterministic simulation inputs beyond supplying the
  current motto to report events.

## Decisions

### Store online profile metadata separately from canonical simulation JSON

Add nullable-or-empty `motto` and `guild` columns through a versioned SQLite
migration and expose them as a credential-safe profile value associated with a
managed character. Registration extracts optional string fields from the
imported document; missing fields become empty strings. Managed reads combine
canonical state and profile metadata for CLI/dashboard inspection.

The worker continues replacing only canonical simulation state. Foreground
profile updates modify only profile columns in their own immediate
transactions, preventing a worker that read an older state from overwriting a
new motto or guild.

Keeping the values only inside `Character` was rejected because
`replace_state` writes the complete serialized character and would introduce a
lost-update race. Mutating only the retained original document was rejected
because worker and dashboard reads intentionally do not expose that
credential-bearing document.

The private original document remains immutable in this change. A future
managed export feature will need to merge current canonical and profile values
into a new browser document rather than exporting the original verbatim.

### Use a second short-lived per-character online-action lock

Retain the existing lifetime character lock exclusively for simulation
ownership. Add a distinct advisory lock used only around preparation and
delivery of manual brags, motto changes, guild actions, and worker-generated
reports.

For worker events, acquire the online-action lock after canonical state has
been persisted, read the current profile, construct the request from the
persisted transition snapshot plus that profile, attempt delivery once, and
release the lock. Foreground actions acquire the same lock without attempting
the simulation lock.

This gives request operations a linear order while allowing profile actions
during active play. Reusing the simulation lock was rejected because it would
make live profile editing impossible. An asynchronous database queue was
rejected because the requirements explicitly avoid durable retries and the
additional worker protocol would add unnecessary failure states.

### Persist motto before its one-shot report

After validation and while holding the online-action lock, persist the new
motto before constructing and delivering `t=m`. Retain it on delivery failure.
This mirrors the browser's local assignment-before-request behavior and ensures
future reports consistently carry the user's selected motto without an
implicit retry mechanism.

The report uses a fresh persisted character snapshot, refreshes Specialty
before construction as existing manual reporting does, and uses the new motto
for the `m` field.

### Persist guild only after a conformance-recognized acceptance

Guild membership is server-authoritative. Deliver `cmd=guild`, consume the
response body internally, and map only browser-observed response forms to safe
accepted, rejected, or indeterminate outcomes. Persist the submitted
designation only for an accepted outcome; an accepted empty designation clears
the current guild.

Persisting before delivery was rejected because a rejected designation would
be displayed locally as membership. Treating every HTTP 2xx as acceptance was
rejected because the browser consumes an application-level message and
optional navigation target from the response body.

The transport must bound response-body size and must never return or log the
raw body. Unknown content becomes an indeterminate safe failure and preserves
the prior guild value.

### Extend disposable conformance before enabling guild delivery

Extend the existing browser harness with a non-empty guild-designation
submission followed by an empty guild-designation submission for its newly
created disposable character. Capture request field names, operation ordering,
browser-visible acceptance/rejection categories, cleanup status, and existing
anti-cheat classification. Do not capture credentials, signed URLs, raw
response bodies, or identifying private-guild text.

Bundled evidence validation gains explicit passing non-empty submission, empty
submission, and cleanup requirements. Production guild actions fail closed
before transport until that evidence is complete. Motto delivery continues to
rely on the existing motto-change trace evidence plus the general
enrollment/reporting gate.

### Expose explicit CLI commands and dashboard editors

Add dedicated CLI commands rather than overloading `report`:

- `motto <id> <text>` sets a motto; `--clear` supplies the explicit empty
  value.
- `guild <id> <designation>` submits the short designation exactly as supplied;
  an empty argument (`""` in a typical shell) leaves the current guild.

The motto command prevents supplying text together with `--clear`. Guild and
motto inputs reject control characters but otherwise preserve Unicode and
spacing exactly.

Add dashboard `m` and `g` actions that open modal editor state initialized from
the persisted value. Printable characters append, Backspace edits, Enter
submits, and Escape cancels. Submitting an empty guild editor requests leaving
the current guild; there is no separate leave key. While an editor is open,
ordinary dashboard hotkeys are interpreted as text.

### Keep dashboard profile actions behind the provider boundary

Extend `DashboardProvider` with motto and guild operations and add profile
values to `DashboardCharacter`. The production provider delegates to the
reporting/profile service; test providers record inputs and return categorized
outcomes. After an action, the dashboard refreshes persisted state before
showing the result message.

The dashboard remains unable to advance or replace canonical simulation state.
Its specification is narrowed so the explicit profile actions may update
profile metadata without violating the existing read-only simulation boundary.

### Apply explicit full-layout sizing and duration formatting

Change the full-layout left-column allocations as follows:

- Activity: four total rows, containing two border rows and the existing two
  content lines.
- Progress: unchanged at seven total rows.
- Details: six total rows, allowing Character ID, formatted elapsed time,
  Motto, and optional Guild.
- Equipment: flexible but capped at thirteen total rows, providing no more
  than eleven inner rows for the eleven equipment slots.

Remove Quest target from Details. Render Motto and Guild independently only
when the corresponding value is non-empty.

Format elapsed seconds with integer division into days, hours, minutes, and
seconds, omit leading zero units, and always include seconds. Examples are
`0s`, `4m 2s`, and `1d 1h 1m 1s`. The formatter is presentation-only and does
not alter stored elapsed values.

## Risks / Trade-offs

- **[Online-action lock held during network I/O]** -> Bound transport timeouts
  and keep the lock separate from simulation ownership; progression continues
  even if a foreground action waits for another request.
- **[A profile action can race with a dashboard refresh]** -> Refresh from
  persistence after the action and treat the persisted profile as the display
  source of truth.
- **[Guild response formats can change server-side]** -> Bound and classify
  only known conformance-derived forms; unknown responses fail closed without
  changing membership.
- **[Imported original documents remain stale after profile edits]** -> Keep
  the original private credential source immutable and document that future
  managed export must merge current state rather than return it unchanged.
- **[Additional Details row reduces flexible space]** -> Recover one row from
  Activity and cap Equipment at its actual maximum useful content height.
- **[Unicode width differs from scalar count]** -> Use the dashboard's existing
  wrapping behavior and keep validation concerned with control characters, not
  terminal display width.

## Migration Plan

1. Upgrade the managed database schema in one transaction, adding motto and
   guild columns with non-null empty defaults so existing rows remain valid.
2. Backfill values from each retained original document when valid optional
   string fields exist; leave empty defaults otherwise. Never expose the
   document or passkey during migration.
3. Deploy readers that tolerate empty profile metadata and writers that update
   profile columns independently from canonical state.
4. Update the worker and explicit reporting paths to use the online-action
   lock and persisted motto.
5. Enable production guild actions only after the bundled conformance evidence
   includes passing non-empty submission, empty submission, cleanup, and
   existing classification checks.

Rollback to an older binary is not supported after the schema version
advances because the current store deliberately rejects newer schemas. Data
rollback can be performed by exporting or copying the database before upgrade;
the migration itself leaves canonical state and retained original documents
unchanged.
