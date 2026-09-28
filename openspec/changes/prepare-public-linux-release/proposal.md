# Proposal

## Why

Gyrognome 1.0.0 is usable locally, but general Linux users currently need a Rust toolchain and manual setup to install it. The private repository has no release pipeline or project license, so opening it or advertising downloads without a publication review would be premature.

## What Changes

- Prepare repeatably built, versioned Linux release archives with `gyro`, the compatibility `gyrognome` binary, the user-service template, and integrity checksums; document supported platforms and an installation/upgrade path that does not require Rust.
- Add a non-publishing release-build workflow with validation of the packaged binaries and service integration. Publishing a GitHub Release remains a separate, explicitly authorized action.
- Document pre-publication review of full Git history and GitHub-hosted material, plus the license and redistribution decision for project and bundled upstream-derived content. A failed or unresolved review blocks a public release; do not silently select a license.
- Clarify that the Node manifest is a conformance-test harness, not a user-facing npm distribution channel.
- Keep public repository visibility changes, GitHub Release publication, crates.io, AUR, `.deb`/`.rpm`, and Flathub outside this change.

## Capabilities

### New Capabilities

- `linux-release-distribution`: Define the downloadable release bundle, installation and service integration, supported targets, integrity checks, and publication-readiness gates.

### Modified Capabilities

None. The existing runtime's user-service and credential-safety behavior remain unchanged.

## Impact

- Release build scripts/workflows, `Cargo.toml` publication metadata (as appropriate), release documentation, service packaging, and any license/attribution files approved after review.
- Build and package verification on Linux; no changes to simulation, network eligibility, user data, service lifecycle semantics, or GitHub repository visibility.
