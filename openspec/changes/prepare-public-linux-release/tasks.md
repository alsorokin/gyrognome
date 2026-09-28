# Tasks

## 1. Publication prerequisites

- [x] 1.1 Inventory tracked files, every reachable Git ref/history, ignored/untracked release inputs, and GitHub-hosted issues/PRs, Actions logs/artifacts, and releases for sensitive or non-redistributable material; verify a private, redacted review record lists each surface checked and all blockers without exposing findings publicly.
- [x] 1.2 Review provenance and redistribution rights for the bundled browser/desktop rule data and conformance material, and obtain the owner's explicit project-license/attribution decision or record publication as blocked; verify the review record identifies each unresolved item rather than assuming permission.
- [x] 1.3 Align Cargo package metadata and the npm test harness's license/package claims with the approved decision (or mark claims unresolved and prevent accidental npm publication); verify the manifests make no unsupported license or npm-distribution claim.

## 2. Candidate bundle preparation

- [x] 2.1 Add a repeatable packaging command for x86_64 and aarch64 Linux that checks requested version, source revision, architecture, and supported target before creating named archives with `gyro`, `gyrognome`, the service template, install instructions, and SHA-256 digests; verify a valid candidate builds with `--locked` and unsupported targets fail explicitly.
- [x] 2.2 Add bundle-content and checksum verification that excludes test-only executables, credentials, player saves, and browser profiles; verify positive bundle checks and negative tests for unexpected files or a modified archive.
- [ ] 2.3 Add an explicitly invoked, least-privilege CI matrix that builds candidates on native x86_64 and aarch64 Linux with a pinned glibc baseline, checks ELF architecture/GLIBC compatibility, and smoke-runs both executables; verify both target jobs produce private candidate artifacts and a failed job prevents a passing candidate status.
- [x] 2.4 Check the workflow for publishing side effects and credential exposure; verify it has no tag-triggered release, visibility change, registry upload, or public asset upload, and that logs/artifact names do not contain sensitive data.

## 3. Installation and documentation

- [x] 3.1 Document or provide an opt-in user-local installation method that verifies the checksum, places both binaries in a chosen user-owned directory, and installs a `systemd --user` template with an absolute path to the installed `gyro`; verify an isolated no-root install works with no Rust/Node toolchain and without relying on service PATH.
- [x] 3.2 Update README/bundle instructions with supported architecture and measured GLIBC floor, direct-worker fallback, service-manager requirements, safe upgrade, backup, rollback, and XDG data preservation; verify each path against an isolated data root and existing migration documentation.
- [x] 3.3 Explain which distribution channels are actually available and identify the Node manifest as test-only; verify README and bundle docs do not imply npm, crates.io, AUR, or `.deb`/`.rpm` installation exists.
- [x] 3.4 Add a maintainer release-readiness checklist requiring completed privacy/provenance review and explicit owner license approval before any separate public-repo or release action; verify an unresolved item yields a visible "blocked" outcome rather than a ready-to-publish recommendation.

## 4. End-to-end validation

- [ ] 4.1 Trial-install both target archives on compatible clean Linux hosts with isolated XDG data; verify checksum checks, `gyro`/`gyrognome` startup, user-service unit path and lifecycle, and stop/upgrade/restart behavior without modifying existing player data.
- [x] 4.2 Run the relevant Rust test suite and Node conformance tests for the candidate source, including fixture-safety checks; verify both suites pass and no candidate artifacts or review records containing sensitive data enter tracked files.
