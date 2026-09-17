## Context

The CLI currently makes every managed-character command target a supplied
stable identifier. The store persists character state, original save data, and
runtime metadata, while lifecycle operations coordinate with a systemd user
service. The dashboard and New Guy wizard already establish terminal-session
and keyboard-interaction patterns. See `proposal.md` for motivation and the
delta specifications for behavioral requirements.

## Goals / Non-Goals

**Goals:**

- Make deletion explicit, credential-safe, and safe against a concurrently
  running character.
- Let dashboard users select a registered character without first running
  `list`, while retaining identifier-based invocation for automation.
- Reuse existing terminal UI patterns and storage/lifecycle abstractions.

**Non-Goals:**

- Bulk deletion, character editing, renaming, export, or database browsing.
- Automatic stopping or recovery of a runtime during deletion.
- Changing character simulation, browser-save compatibility, or network
  boundaries.

## Decisions

### Keep character identity as the selection and deletion key

Add a `delete <id>` CLI operation and make `dashboard`'s identifier optional.
The deletion command will show the resolved credential-safe identity and use a
standard-input explicit confirmation; the dashboard's absent-ID path will
reuse the terminal interaction stack for a keyboard-navigable picker. This
keeps direct commands stable for scripting and does not require an ambiguous
name lookup. A name-based deletion command was rejected because names need not
be unique.

### Gate deletion on lifecycle ownership before store mutation

The administration flow will check the existing local lifecycle/ownership
state before invoking a single store-level removal operation. It will reject an
active or owned character rather than stopping it automatically, preventing a
worker from persisting state after deletion. The alternative of automatically
stopping the service risks treating a failed stop as a successful deletion
precondition and makes an irreversible operation less explicit.

### Make removal atomic at the persistence boundary

The store will own deletion of all records for one character within its
existing transactional persistence mechanism. The caller will not delete
individual files or database records. This preserves the existing durability
contract and allows errors to leave the last complete character available.

### Separate the picker from dashboard rendering

The selector will resolve one `CharacterId` or a cancellation before the
dashboard's current refresh/render loop starts. It will list only identity
fields already permitted by `list`, have an empty-state error, and restore the
terminal on cancel or failures. Embedding selection into the live dashboard
would complicate refresh and lifecycle state while providing no benefit for
the entry-only flow.

## Risks / Trade-offs

- [A service can start after the pre-deletion status check] → Recheck
  ownership within the deletion path and make the store mutation conditional
  on no active owner where the existing locking model permits it.
- [Interactive confirmation limits non-interactive automation] → Preserve the
  explicit deletion command and add no bypass in this change; safety is more
  important for destructive local administration.
- [Terminal setup can fail while selecting] → Reuse the established terminal
  session cleanup path and surface the error without changing persisted data.

## Migration Plan

No persisted schema migration is expected. Existing registrations remain
unchanged and become selectable by the dashboard. Deploying the new binary
adds the deletion command; rollback simply removes the command while already
registered data remains readable by the prior version.
