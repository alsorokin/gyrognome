# Design

## Context

See [proposal.md](proposal.md) for motivation and [the new capability spec](specs/linux-release-distribution/spec.md) for observable requirements. `Cargo.toml` builds two normal executables (`gyro` and `gyrognome`) and one feature-gated conformance binary. `systemd/user/gyrognome@.service` currently uses `ExecStart=gyro worker %i`, which depends on the user manager's PATH. README instructions assume a source checkout; `package.json` exists to run Playwright/Node conformance tests and claims ISC although no project license file exists. `.gitignore` excludes `saves/` and local browser profiles, and fixture-safety tests cover committed fixtures but not historical refs or GitHub-hosted content. There is no existing Actions workflow or GitHub Release.

## Goals / Non-Goals

**Goals:**
- Make a candidate Linux bundle installable by an ordinary user without a compiler or a privileged package manager, preserving the same per-user data and lifecycle semantics.
- Make release candidate construction and platform verification repeatable and non-publishing.
- Make review blockers visible before a maintainer separately decides whether to publish.

**Non-Goals:**
- Guarantee bit-for-bit reproducible builds, promise support for all Linux distributions, or alter runtime/reporting behavior.
- Change repository visibility, push tags, upload GitHub Releases, publish crates/npm/AUR packages, or automatically rewrite Git history.

## Decisions

### Build two glibc targets as private candidate artifacts

Use `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`, built on compatible native Linux CI runners in a pinned Ubuntu 22.04 build environment with `cargo build --release --locked --bins` and the default feature set. Compare actual ELF architecture and required GLIBC symbol versions against the documented compatibility baseline (targeting glibc 2.35 or older); fail rather than claim support if the runner/container cannot build or test a target. Include a target-labeled `tar.gz` with `gyro`, `gyrognome`, the existing service template, a small install guide, and, once legally approved, the chosen license/attribution. Supply a SHA-256 digest file for each archive. Rust SQLite is bundled and HTTP uses rustls, but system libc compatibility still needs verification. Pin source revision, lockfile, action versions, and build environment; do not claim deterministic archive bytes without testing that separately.

Alternative: musl/static releases could cover more distros but require specific compatibility validation and extra build/toolchain work; `.deb`/`.rpm` multiply packaging surfaces. Neither is the first release candidate.

### Require an explicit manual preparation trigger

Use a `workflow_dispatch` matrix, not tag-triggered publishing. Produce private workflow artifacts; validate archive contents, exclusions, executable `--help` or `--version` behavior, checksum verification, and service configuration per target. Fail the matrix if either target is unavailable. An operator will publish separately only after review. Keep the workflow's permissions minimal and avoid external upload endpoints or a privileged release token.

Alternative: a full `dist`-generated publish workflow automates distribution but introduces unrequested public side effects and packaging complexity; it can be reconsidered later.

### Install in a user-owned directory with an absolute service path

Document verifying the downloaded SHA-256 digest and extracting the archive before copying executables into `~/.local/bin` (or a user-chosen absolute path). Provide a small opt-in user-local installer or a carefully tested command recipe that writes the user unit with an absolute `ExecStart` path to the installed `gyro` binary, reloads `systemctl --user`, and does not enable/start services automatically. Ensure the documented CLI invocation works even when `~/.local/bin` is absent from interactive PATH. A worker can be run directly where a user service manager is unavailable; `gyro start/status/stop/recover` continue to require it. Upgrades stop active workers, back up XDG state, replace binaries, then restart only the previously running workers; rollback follows the existing documented pre-migration snapshot rule.

Alternative: leaving the unit's relative `ExecStart=gyro` in place is convenient for local source builds but unreliable for an archive installed under the home directory.

### Treat visibility and license as human approval gates

Prepare a private maintainer checklist: inspect all refs and history, relevant GitHub issues/PR attachments, Actions history/logs/artifacts (if present), release assets, and tracked content; inspect bundled rule snapshots and conformance evidence for redistribution rights; record findings without reproducing secrets in public logs. Obtain an owner-selected project license and resolve attribution/rights before making any public claims; align Cargo and npm metadata with that decision, without interpreting the npm manifest as an npm product. If approval is not obtained during implementation, keep the candidate internal and mark publication blocked instead of making up an SPDX license. Re-check ignored/untracked files and release archive contents before a manual publish decision.

Alternative: relying on `.gitignore`, current-tree scanning, or fixture-safety tests alone misses history, remote attachments and legal provenance.

## Risks / Trade-offs

- [Private GitHub Actions artifacts are accessible to authorized collaborators and have retention limits] -> Keep candidates free of credentials, verify archive contents, use least privilege, and document that CI artifacts are not a permanent public download.
- [Runner architecture or oldest-supported GLIBC differs from expectations] -> Gate each target on a native smoke run and actual ELF checks; update support claims only from measured results.
- [Local service points at the wrong executable after installation/upgrade] -> Validate the installed unit's absolute path with a user-scoped service test and preserve the old worker behavior.
- [History contains material requiring rotation or removal] -> Stop publication; rotate exposed credentials, handle history/remote artifacts separately with owner approval, and repeat the review.
- [Bundled upstream material cannot be redistributed on the assumed terms] -> Do not publish until rights and attribution are explicitly resolved; a project license alone cannot license someone else's work.

## Migration Plan

1. Add non-publishing candidate packaging, CI checks, and docs while the repository remains private; trial-install bundles on clean x86_64 and aarch64 Linux hosts with isolated XDG data.
2. Record the publication review and license/attribution decision; if unresolved, leave the candidate unpublished and report blockers.
3. A later, separately authorized change may make the repository public and upload a reviewed candidate to a GitHub Release. Until then, there is no user-facing migration.
4. If a candidate install fails, stop affected user workers and restore the previous executable/unit; if state was migrated, use the existing private pre-migration backup procedure before running an older binary. Do not delete or rewrite the XDG character store as part of packaging.
