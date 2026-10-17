# Tasks

## 1. Make last accessed dashboard-only

- [ ] 1.1 Repurpose the existing persisted timestamp as dashboard-access metadata, initialize browser/desktop registrations to never opened, and reset legacy worker-derived values through the next schema migration; verify fresh-store and upgrade tests, persistence after reopening, and unchanged canonical state, profile values, rested data, and worker timing baselines.
- [ ] 1.2 Remove access-timestamp updates from browser/desktop worker persistence, profile edits, and other non-dashboard writers, and add a typed dashboard-access write method; verify regression tests show those writes preserve access time, including a worker record loaded before the dashboard records an opening.

## 2. Wire resume and successful-opening tracking

- [ ] 2.1 Update selector recency to use dashboard-only access times, render never-opened entries as `Never`, and share its timestamp-descending/stable-ID-ascending ordering with resume; verify selector tests cover opened-first ordering, equal-time ties, and independence from worker/profile updates.
- [ ] 2.2 Add `--resume`/`-r` to `Command::Dashboard` and open the first previously opened registered character in that ordering, reusing the selector when none exists; verify both aliases, ID conflicts, refresh compatibility, unchanged plain invocation, next-most-recent selection after deletion, never-opened fallback, empty-store errors, and genuine storage-error propagation.
- [ ] 2.3 Add the provider opening notification and record access once after the first successful dashboard draw; verify focused dashboard tests cover notification ordering, no timestamp change on failed startup or cancellation, no repeat on refresh, and explicit write-error propagation through terminal cleanup.
- [ ] 2.4 Exercise the full command in an isolated pseudo-terminal using synthetic browser and desktop characters: open B then A by ID and selection, exit, update B, and resume with both flag forms; verify A remains first in the selector and opens directly, deleting A makes resume open B, and removing all previously opened characters makes resume show the selector. Do not start workers or submit network requests.

## 3. Document and check

- [ ] 3.1 Update README dashboard usage with both flag forms, shared last-dashboard recency, `Never` and upgrade behavior, fallback when no previously opened character remains, and ID exclusivity; verify the text agrees with `gyro dashboard --help`.
- [ ] 3.2 Run `cargo fmt --check`, focused runtime/CLI/dashboard unit tests, and `cargo test --test local_character_runtime dashboard`; verify all pass without live services or real saves and fix only failures related to this change.
