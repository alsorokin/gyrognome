# Proposal

## Why

Pemptus transport and simulation already work, but overly narrow evidence gates
split normal use across incompatible import paths. Finish the practical feature:
supported Pemptus characters such as Izot can progress, brag, edit their motto,
and join/change/leave guilds without becoming local-only merely for being Pemptus.

## What Changes

- Treat unadapted, spelling-only, and quest-placeholder-only Pemptus imports as
  a narrowly approved equivalent behavior family, using existing live records
  and completed source/synthetic checks for level, act, manual, and motto gates.
  Retain each record's actual observed path, integrity, and freshness.
- Enable Pemptus guild actions using one native request and bounded public
  membership confirmation. Missing historical response fingerprints no longer
  prevent the action; an unconfirmed result never commits optimistic membership.
- Use the same readiness decisions in inspection, managed runtime, worker/manual
  reporting, profile actions, and dashboard. Already imported, unadvanced records
  gain support without reimport or source edits.
- Preserve exact realm/authentication contracts, private data handling, native
  callback ordering, no retries, irreversible local-only history, and existing
  Spoltog/browser behavior. Unsupported or combined adaptations remain excluded.
- Deliver with focused regressions and clear documentation, not another long
  live-validation campaign.

## Completion Scope

Otherwise eligible imports on each supported path must expose all five
operations. Automatic progress must not create a local-only fork solely because
of the former path mismatch; guild state changes only after public confirmation.

This deliberately replaces the earlier partial-delivery and per-path live-proof
policies with a pragmatic readiness contract. It is usable imported-character
support, not an exhaustive anti-cheat/conformance certification. Missing
historical guild fingerprints remain missing. No fixture is relabeled or
invented, and this planning revision authorizes no live experiment or replay.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `opt-in-leaderboard-reporting`: Common Pemptus readiness across the supported
  equivalent paths, including automatic reporting for existing imports.
- `online-character-profile`: Pemptus motto and publicly confirmed guild actions.
- `leaderboard-conformance`: Pragmatic readiness using preserved evidence and
  explicit equivalence, separate from exhaustive live certification.
- `terminal-dashboard`: Consistent usable-operation availability and safe gates.

## Impact

Existing eligibility/evidence selection, guild verification, inspection/runtime/
dashboard presentation, focused tests, README, and desktop compatibility docs.
No new dependencies, database migration, simulator, or mandatory live campaign.

## Non-goals

Native desktop enrollment or character creation, other realms or endpoint
aliases, Pemptus account/password support, non-ASCII desktop encoding, new
simulation rules, recovery of historical RNG, queued/retried reports, or
reconnecting an already local-only managed timeline.
