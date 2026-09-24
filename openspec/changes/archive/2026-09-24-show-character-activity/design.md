## Context

See `proposal.md` for motivation. The current `dashboard` command lists registered characters with `Store::list()`, narrows each entry to `(CharacterId, CharacterIdentity)`, and passes that to `dashboard::select_character`. `Store::list()` currently orders by creation time even though `ManagedCharacter` already carries `updated_at_unix_ms`, and the selector renderer only shows identity plus id. Runtime activity is already available through the local lifecycle/status path used by the dashboard and status command; selection must remain credential-safe and local-only.

## Goals / Non-Goals

**Goals:**

- Carry enough credential-safe metadata into the selector to render identity, id, last accessed time, and active/inactive state.
- Sort selector entries by last accessed time descending before user interaction begins.
- Reuse existing lifecycle/service status behavior so "active" matches the rest of the dashboard.
- Keep selection cancellation and no-registration behavior unchanged.

**Non-Goals:**

- Do not change the full dashboard layout after a character is opened.
- Do not add a new database table, background refresh loop, or network request.
- Do not expose browser save contents, passkeys, raw endpoint responses, or unrecognized raw save fields.

## Decisions

1. **Use existing managed-character update metadata as the selector's last accessed value.** `updated_at_unix_ms` is already persisted and loaded with each `ManagedCharacter`, and it changes when managed state/profile data is updated. This avoids a schema migration and keeps ordering tied to the same local persistence boundary as the dashboard. Alternative considered: add a new "last opened in dashboard" field, but that would require writes from a selection screen that is otherwise read-only and would change cancellation behavior.

2. **Introduce a selector-specific metadata type instead of expanding tuple usage.** Replace the `(CharacterId, CharacterIdentity)` selector input with a named structure carrying id, identity, last-access timestamp, and active status. This keeps rendering and navigation code explicit while preserving the selected-index behavior. Alternative considered: pass `ManagedCharacter` directly, but that would expose more state than the selector needs and make privacy review harder.

3. **Resolve active state before entering terminal selector mode.** The CLI should collect registered characters, query the local lifecycle status for each candidate, sort the resulting selector entries by last access descending, and then enter the selector. This avoids mutating selector state while the user navigates and keeps service-status failures out of the rendering loop. Alternative considered: refresh active status continuously inside the selector, but that adds polling and redraw complexity beyond the requested pre-selection context.

4. **Display service-status failures as non-active/unknown metadata without blocking selection.** A failure to determine one character's service status should not prevent selecting another character. The opened dashboard already reports refresh failures in its status area. The selector should show a credential-safe unavailable indicator for status lookup failures while still satisfying sort-by-recency behavior. Alternative considered: fail the entire selector on any status lookup failure, but that would make the dashboard less available than today.

## Risks / Trade-offs

- **Status collection scales linearly with registered characters** -> Use the existing bounded local status checks and keep them outside the render loop; revisit only if very large character lists become a supported use case.
- **`updated_at_unix_ms` means last persisted access/update, not merely last highlighted selection** -> Label and tests should align with the existing persistence meaning; avoid adding write-on-view semantics in this change.
- **Runtime status can change after selector render** -> Treat selector activity as a point-in-time hint; the opened dashboard remains the authoritative refreshed view.
- **Terminal width may truncate richer entries** -> Keep each entry concise and preserve the id line so selection remains unambiguous.
