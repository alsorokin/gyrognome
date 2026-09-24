## Context

See `proposal.md` for motivation. The dashboard Brag action and confirmed CLI
`report` command both call `reporting::submit`, so the failure is in the shared
reporting boundary rather than either presentation layer.

`Store::reporting_target` currently acquires the character ownership lock
before resolving the persisted state and retained credential. That ownership
lock correctly excludes a second worker, but it also rejects foreground manual
bragging whenever the normal worker is active. Motto and guild actions instead
use a per-character online-action lock that serializes network operations while
leaving worker ownership intact. Worker-generated reports acquire the same
online-action lock through their worker-specific target path.

## Goals / Non-Goals

**Goals:**

- Reuse the existing online-action boundary for foreground manual brags.
- Preserve one coherent persisted state/profile snapshot and credential for
  request construction.
- Serialize manual brags with motto, guild, and automatic worker report
  deliveries for the same character.
- Preserve active worker ownership and all existing reporting safety gates.

**Non-Goals:**

- Do not stop, restart, recover, or communicate directly with the systemd
  service.
- Do not include dashboard-predicted progress or unpersisted worker state in a
  report.
- Do not add retries, queues, new endpoints, protocol changes, or persistence
  schema changes.
- Do not change confirmation behavior or safe outcome wording.

## Decisions

### Use the existing online-action target for manual bragging

`reporting::submit` will resolve its snapshot through the same
active-runtime-safe target used by profile actions. The target acquires the
per-character online-action lock, reads the persisted character/profile row,
and resolves the retained passkey before request construction and delivery.
The lock remains held for the target lifetime, so the entire online operation
is serialized.

This is preferred over temporarily stopping the worker because lifecycle
changes would be user-visible, failure-prone, and unnecessary for a read-only
report. It is also preferred over bypassing all locks because simultaneous
manual, profile, and automatic reports could then race at the endpoint.

### Keep worker ownership separate from online-action serialization

The worker ownership lock will continue to protect exclusive simulation
ownership only. Foreground online actions will not acquire it. The worker may
persist a successor after a foreground report captures its database snapshot;
the report still represents one coherent persisted state at the point it was
read, which matches the existing foreground-report contract.

This is preferred over making worker persistence acquire the online-action
lock because that would couple simulation progress to potentially slow network
delivery and expand the change beyond online request serialization.

### Remove the obsolete inactive-only target

After `reporting::submit` switches to the online-action target, the
inactive-only `Store::reporting_target` path and the optional ownership-lock
field in `ReportingTarget` have no production caller. They will be removed or
consolidated so the runtime API no longer encodes the superseded restriction.
The worker-specific target remains because automatic reports use the persisted
worker event snapshot while resolving current credential-safe metadata.

This is preferred over leaving duplicate target methods because duplicate
credential extraction and stale comments would make the new ownership rules
unclear and easier to regress.

### Test the shared boundary and both user surfaces

Reporting tests will hold an active `Worker`, submit one manual brag with a
recording transport, and verify one delivery plus continued ownership and
unchanged managed state. Runtime tests will cover the consolidated
online-action target for active, inactive, offline, unknown, and invalid
credential cases. CLI integration coverage will confirm that an explicitly
confirmed report succeeds while active; dashboard tests will retain the
single-call and safe-message assertions because its provider delegates to the
same shared function.

## Risks / Trade-offs

- **A report can be immediately superseded by the next worker persistence**
  -> Define the report as using the coherent persisted snapshot captured after
  acquiring the online-action lock; do not claim it includes unpersisted or
  predicted progress.
- **A slow endpoint holds the online-action lock and delays another online
  action** -> Retain the existing bounded transport behavior and one-attempt
  delivery policy; do not block simulation persistence or worker ownership.
- **Removing the ownership lock could accidentally weaken credential or
  endpoint checks** -> Change only target acquisition and retain conformance
  validation, online credential validation, official endpoint validation, and
  credential-safe result handling unchanged.
- **CLI and dashboard behavior could diverge later** -> Keep both surfaces
  delegated to the shared `reporting::submit` function and cover the shared
  active-worker behavior directly.

## Migration Plan

No data or service migration is required. Deploy the updated binary normally;
existing workers continue running, and subsequent dashboard or CLI manual
brags use the new serialized boundary. Rollback restores the previous refusal
behavior without changing persisted data.
