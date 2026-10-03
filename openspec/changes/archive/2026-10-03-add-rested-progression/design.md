# Design

## Context

See proposal.md for motivation and the delta specs for behavior.

`src/runtime.rs` owns the SQLite store, exclusive character lock, and both worker loops. Browser workers currently cap elapsed inputs and commit every advancement; desktop workers use `DesktopSession`, commit at completion/report/provenance/stop points, and otherwise hold partial bars in memory. The desktop simulation caps callback inputs at 100 ms and dispatches a full bar only on a subsequent callback.

The existing `updated_at_unix_ms` also changes on profile updates, so it cannot represent rested accounting. The database already supports transactional schema migration and private pre-migration backups. Dashboard `TaskAnchor` currently predicts only from profile-specific rates, with display-only prediction and independent state/status refreshes.

Two existing policies need explicit revision, not accidental bypass: `Controlled offline advancement` prohibits production callback acceleration, and classic evidence policy invalidates conformance-relevant behavior changes. The runtime delta permits rested pacing; the leaderboard delta narrowly preserves otherwise valid production operation readiness for this timing change while leaving live experimental procedures unaccelerated.

## Goals / Non-Goals

**Goals:** Separate real active time, suspend time, and simulation time; retain deterministic profile transitions; persist bank spending with the progress it paid for; integrate with existing inspection and task prediction.

**Non-Goals:** Changing reward tables or canonical save schemas, changing desktop completion semantics, detecting server anti-cheat policy, adding a background reporting queue, or introducing configuration/opt-in controls.

## Decisions

### 1. Keep the bank in managed runtime metadata

Add a small rested metadata record per managed character containing balance, dedicated accounted wall-time baseline, and whether its last checkpoint represented an active or stopped session. Keep it separate from official canonical state, profile text, import provenance, and `updated_at_unix_ms`. Extend existing SQLite transactions rather than creating a second persistence system.

Use integer durations with sub-millisecond precision internally, retaining fractional time across 54.6875 ms desktop periods. Persist that precision (for example, balance in integer nanoseconds) and expose whole remaining milliseconds in safe inspection. Do not repeatedly truncate callback-sized spending to whole milliseconds.

Registration initializes zero at registration time. Migration initializes existing rows to zero at migration time, not at their old update timestamps. Read-only inspection projects pending stopped accrual or active spending from metadata and current ownership without writing it back. Its projected active countdown is an estimate; the worker's monotonic accounting is authoritative.

Alternatives rejected: canonical save fields would affect interchange/conformance; general update timestamps would incorrectly reset rest on profile changes.

### 2. Use Linux clocks to distinguish suspend from delays

During an active session, sample `CLOCK_MONOTONIC` and `CLOCK_BOOTTIME`. The latter includes suspend/hibernate while the former excludes it; positive growth in their difference is sleep duration. Use monotonic awake elapsed for spending and normal pacing. Do not use a large wall-clock gap as a sleep heuristic.

Use a small fallible clock adapter with injected samples for tests. A direct Linux clock-access dependency such as `libc` is sufficient; no logind daemon integration is needed. Clock failures propagate through the existing safe runtime error boundary.

Across stopped sessions, use the dedicated wall-clock baseline, since neither monotonic clock supplies cross-boot elapsed time. Clamp accrual at the bank cap, retain an accounted wall-time high-water mark across backwards clock movement, and do not let profile reads/edits replace that baseline.

On wake, credit the independently measured sleep once, retain the pre-sleep bank without spending it, and reset progression deadlines so resume cannot replay missed callbacks. Account for any legitimate awake interval separately rather than crediting the entire sleep gap as progression.

Alternatives rejected: wall-clock-gap detection rewards awake scheduler/network stalls; suspend notifications require unnecessary event-service plumbing.

### 3. Spend real active time; accelerate only eligible progression

The bank measures future real-time boosted duration. For a normally serviced active interval `a` and initial balance `r`:

```text
boosted_real = min(r, a)
simulated    = a + boosted_real
remaining    = r - boosted_real
```

If the worker is delayed, charge the bank for actual awake session time, including processing and delivery, but cap game advancement using the existing profile delay policy. Spending does not create a deferred progression entitlement. Sleep is neither spending nor an immediate simulation input.

Split timing at bank exhaustion. Keep the runtime's scheduling/accounting helper pure over explicit clock samples and durations, leaving `simulation::advance_with_trace` and desktop callback transitions unchanged. Use precise deadlines/remainders so repeated short updates do not invent or lose elapsed contributions.

Alternative rejected: doubling XP rewards would leave tasks, loot, quests, and plots at normal speed and would not implement the agreed mechanic.

### 4. Accelerate browser elapsed inputs and desktop virtual pacing

For browser workers, keep `--interval-ms` as the maximum real-time contribution. Schedule at the earliest of tick-aligned task completion at the current speed, bank exhaustion, the interval, and the accounting checkpoint deadline. Apply the speed split to the bounded contribution; a one-second interval can earn up to two virtual seconds. Service only complete 100 ms virtual ticks and persist the fractional remainder. For example, one second with 250 ms rest earns 1,250 virtual ms, services 1,200 ms, and retains 50 ms.

Implementation discovery confirmed that the existing simulation discards tick overshoot at a task boundary: advancing a 450 ms task by 500 ms differs from advancing by 450 ms then 50 ms. The user approved preserving those semantics rather than changing the simulation core. Stable virtual ticks therefore replace the original plan's arbitrary-partition equivalence claim; scheduling must preserve tick sequences, not round a short interval up or dispatch partial ticks.

For desktop workers, run the existing callback engine on a piecewise 1x/2x virtual clock. Its 109.375 ms virtual callback period is 109.375 ms real at 1x and 54.6875 ms real at 2x. Integrate elapsed input over the correct speed portions before passing it through the unchanged 100 ms cap. Both progression callbacks and completion-only callbacks follow the faster cadence, preserving their ordering.

Use future deadlines and skip missed periods at both speeds. Recalculate scheduling when rest changes or expires; do not run a burst to settle virtual-clock debt. Desktop measured-since-import counters remain credited game milliseconds, not newly fabricated lifetime history.

Alternatives rejected: doubling desktop inputs alone hits the cap; changing task maxima/rewards alters canonical semantics; synthesizing callback batches after a delay violates the no-catch-up contract.

### 5. Commit accounting with progress and lifecycle transitions

Extend browser state replacement and `replace_desktop_checkpoint` to commit rested metadata in the same transaction as their canonical/random/provenance data. Persist settled rest on startup after taking the character lock, without advancing or reporting. Graceful stop must settle and commit accounting for both profiles, including an otherwise clean or completion-free session.

Keep the existing desktop event/provenance commit points and add a periodic checkpoint at most one awake second apart while servicing callbacks. This persists partial state and bank together instead of adding a separate per-callback bank write. Browser scheduling also observes the accounting checkpoint bound, even when its configured interval is longer. Ordinary reads and profile updates must not overwrite this metadata.

A crash restores the last complete checkpoint. Its actual stop instant is unavailable, so subsequent rest is estimated from that checkpoint; no exact crash-time claim is made. Normally serviced checkpoints bound this ambiguity to about one second, but a crash during a long blocked operation can leave a larger unknown tail. This limitation is preferable to a second heartbeat process or runtime event journal.

Alternatives rejected: independent bank writes can restore spent rest alongside committed boosted progress; completion-only desktop accounting leaves long periods stale.

### 6. Reuse safe presentation and profile prediction

Add a credential-free rested view to managed human/JSON inspection and `DashboardCharacter`. Expose available milliseconds and active multiplier; inactive characters can have a bank but are not actively boosted.

Carry observed remaining rest into `TaskAnchor`. Predict with the profile's normal rate plus one extra normal-rate contribution for the portion covered by observed rest. Include the exhaustion boundary in prediction and redraw/read-deadline calculations. Re-anchor on newer timing metadata even if the task identity has not changed.

Put concise rested status in existing full Details and compact Character content so fixed pane heights do not change. Details and each character-selection entry show only `Rested: <time>` using the same read-only projection, without "available" or a multiplier; compact Character content retains its multiplier. Omit technical ID and compatibility-profile rows from Details. Keep normal task-only prediction, ownership checks, saturation, settling reads, and terminal scrolling. Display estimates never persist accounting or create game transitions.

### 7. Preserve existing online gates without claiming boosted acceptance

The approved exception covers only production rested timing. Do not change request fields, validators, credentials, endpoints, operation-specific gates, provenance, evidence files, or report ordering. Preserve current passing operation evidence/contract identities when their simulation/protocol behavior is unchanged; do not relabel it as rested proof.

If a desktop character is already gated, its first actual advancement still follows existing local-only provenance rules. Rest alone does not confer or remove eligibility. Existing development conformance runners remain at original speed and outside managed rested accounting.

README and managed inspection state that rested-timeline acceptance/classification is unverified; dashboard views omit that long notice. No live verification commands or automated acceptance campaign belong in implementation tasks. The user will test their own characters separately.

## Risks / Trade-offs

- [Official servers may reject or classify boosted timelines differently] -> Disclose the uncertainty; do not claim acceptance or add a verification gate the user declined.
- [Crash timing has an unobserved tail, especially during blocked delivery] -> Recover only complete checkpoints, use periodic serviced checkpoints, and document the downtime estimate rather than fabricate an exact stop.
- [Forward wall-clock changes can credit stopped time that was not real] -> Cap the bank at 12 hours; accept this local-clock limitation rather than add a trusted-time service. Backwards movement retains a high-water baseline.
- [Faster desktop pacing increases callback and commit pressure] -> Retain in-memory partial callbacks and bounded checkpoints instead of writing every callback; skip late deadlines rather than repay them.
- [Display prediction can differ during stalls, sleep, or clock adjustments] -> Treat it as an estimate, re-anchor on refresh, and never use it for persisted progress or eligibility.

## Migration Plan

1. Add the next database schema migration using the existing private backup and transactional initialization pattern. Initialize every existing character's empty bank and baseline together; new registrations initialize equivalent metadata.
2. Deploy updated worker and presentation code together. Restart old workers during upgrade so they do not continue committing without rested metadata.
3. Preserve canonical state, identifiers, random continuation, private credentials, and existing provenance through migration and character removal. Rested metadata is deleted with its character.
4. Older binaries reject the newer schema as they already do. A rollback needs an explicit restore from the pre-migration backup and loses post-migration progress; do not automatically downgrade or overwrite live state.
