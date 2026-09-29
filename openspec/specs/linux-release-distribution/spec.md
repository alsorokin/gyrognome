# linux-release-distribution Specification

## Purpose

Enable Linux users to install Gyrognome without a Rust toolchain while keeping downloads, local user-service integration, and publication decisions explicit and verifiable.

## Requirements

### Requirement: Versioned Linux download bundles

The release preparation process SHALL build separately labeled Linux x86_64 and aarch64 bundles from a specified source revision and version, containing runnable `gyro` and compatibility `gyrognome` executables, the `gyrognome@.service` user-service template, and installation instructions. Each bundle SHALL declare its architecture, Linux C library compatibility, version, and SHA-256 digest. Test-only executables, player saves, private credentials, and browser profiles MUST NOT be included.

#### Scenario: Inspecting a prepared bundle
- **WHEN** a Linux release bundle is produced for a supported target
- **THEN** its contents identify the target and version, include both normal executables and the service template, and provide a digest that verifies the archive bytes

#### Scenario: Preparing an unsupported target
- **WHEN** a requested target is not one of the documented supported Linux targets
- **THEN** release preparation fails with an explicit unsupported-target error instead of labeling an untested binary as supported

### Requirement: Toolchain-free installation and safe upgrade

The distribution instructions SHALL enable an unprivileged Linux user to verify and install a compatible bundle without Rust or Node.js, find the interactive `gyro` command, and configure the installed user service to invoke that same binary by a reliable absolute path. The instructions SHALL explain service-manager prerequisites, a direct worker option when `systemd --user` is unavailable, how to upgrade or roll back a binary while preserving the existing XDG character store, and when a database backup is required.
The instructions SHALL distinguish verified native service lifecycle coverage from architecture-specific installation trials under emulation.

#### Scenario: Installing as an ordinary user
- **WHEN** a user follows the bundle's installation instructions on a documented compatible host
- **THEN** `gyro` runs without a Rust toolchain or root privileges and the service uses the installed executable rather than relying on the user's interactive shell PATH

#### Scenario: Upgrading a managed installation
- **WHEN** an existing user prepares to replace a Gyrognome binary
- **THEN** the documented steps preserve the XDG store and explain stopping/restarting active workers, backing up data, and the limits of restoring an older binary after a schema migration

#### Scenario: Reading ARM support claims
- **WHEN** a user checks validation coverage for the aarch64 bundle
- **THEN** the instructions disclose whether its user-service lifecycle has been exercised on a native ARM host, separately from binary smoke tests and emulated installation

### Requirement: Non-publishing release verification

The repository SHALL offer a repeatable, explicitly invoked release-preparation workflow that checks the version/target labels, builds the proposed bundles and checksums, verifies their contents, runs an executable smoke check on each supported target, and retains temporary artifacts for review. Documentation SHALL disclose that artifacts from a public repository are available to signed-in users with read access and distinguish them from durable release downloads. The workflow SHALL NOT change repository visibility, create a GitHub Release, or publish to any package registry.

#### Scenario: Successful candidate build
- **WHEN** a maintainer invokes the release-preparation workflow on a supported source revision
- **THEN** the workflow produces temporary reviewable bundles and checksum files without creating a durable GitHub Release or package publication

#### Scenario: Failed candidate verification
- **WHEN** a required target build, checksum, contents check, or executable smoke check fails
- **THEN** the workflow reports failure and does not present that candidate as release-ready

### Requirement: Explicit publication-readiness review

The repository SHALL document a review of reachable Git history and GitHub-hosted Actions logs/artifacts and other repository material for sensitive or non-redistributable content. Publication guidance SHALL require an explicit owner-approved project license and resolution of rights for bundled upstream-derived material, with consistent manifest license claims and attribution where applicable. Missing review evidence, unresolved rights, or discovered sensitive material MUST be reported as blockers rather than treated as successful publication readiness. Test-fixture validation alone SHALL NOT count as the full-history review.

#### Scenario: Unresolved licensing
- **WHEN** the project license or redistribution rights for bundled content remain undecided
- **THEN** publication guidance identifies the unresolved decision as a blocker and does not recommend making the repository or archives public

#### Scenario: Sensitive content discovered in history
- **WHEN** the pre-publication review finds a credential, raw save, private artifact, or other sensitive content in any exposed material
- **THEN** the review records a blocker and directs the maintainer to remediate and re-review before changing visibility

### Requirement: Clear distribution-channel labeling

Public-facing installation guidance SHALL distinguish downloadable Linux archives from Rust source installation and the Node.js conformance-test harness. It SHALL NOT advertise npm, crates.io, AUR, or distro package feeds as available Gyrognome distribution channels unless independently published.

#### Scenario: Selecting an installation route
- **WHEN** a Linux user reads the prepared installation guidance
- **THEN** they can identify the supported downloadable archive and do not mistake the existing npm manifest for a working npm installer
