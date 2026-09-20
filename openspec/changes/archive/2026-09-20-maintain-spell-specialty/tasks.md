# Tasks

## 1. Canonical Specialty derivation

- [x] 1.1 Replace the last-reward Specialty assignment with the official indexed-rank selection rule, including earlier-spell tie behavior and an empty result for no spells; verify focused deterministic tests.
- [x] 1.2 Recompute canonical Specialty before native persistence and automatic or explicit report snapshots, and verify existing learned spells can produce a reportable Specialty without another reward.

## 2. Reporting regression coverage

- [x] 2.1 Add deterministic coverage for indexed-rank selection, equal products, and empty spell collections; update only the affected canonical checkpoint and synthetic trace fixtures.
- [x] 2.2 Add automatic and manual report coverage proving that the derived Specialty is emitted as the existing `z` request field, then run the full Rust and Node leaderboard-conformance suites.
