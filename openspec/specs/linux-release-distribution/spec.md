# linux-release-distribution Specification

## Purpose

Enable Linux users to install Gyrognome without a Rust toolchain while keeping downloads, local user-service integration, and publication decisions explicit and verifiable.

## Requirements

### Requirement: Versioned Linux download bundles

Release preparation SHALL build separately labeled x86_64 and aarch64 Linux bundles from a specified revision and version. Each bundle SHALL contain runnable `gyro` and compatibility `gyrognome` executables, the user-service template, and installation instructions; declare its architecture, libc compatibility, version, and SHA-256 digest; and exclude test-only executables, player saves, private credentials, and browser profiles.

#### Scenario: Inspecting a prepared bundle
- **WHEN** a Linux release bundle is produced for a supported target
- **THEN** its contents identify the target and version, include both normal executables and the service template, and provide a digest that verifies the archive bytes

#### Scenario: Preparing an unsupported target
- **WHEN** a requested target is not one of the documented supported Linux targets
- **THEN** release preparation fails with an explicit unsupported-target error instead of labeling an untested binary as supported

### Requirement: Toolchain-free installation and safe upgrade

Installation instructions SHALL enable unprivileged bundle verification and installation without Rust or Node.js, expose the `gyro` command, and configure the user service with its absolute binary path. They SHALL document service-manager prerequisites, a direct-worker fallback, upgrade and rollback while preserving the XDG store, backup needs, and distinguish native lifecycle tests from emulated architecture trials.

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

The repository SHALL provide an explicitly invoked release-preparation workflow that labels targets and versions, builds bundles and checksums, verifies contents, runs executable smoke checks on supported targets, and retains artifacts for review. Documentation SHALL explain access to artifacts from a public repository and distinguish them from durable downloads. The workflow SHALL NOT change repository visibility, create a GitHub Release, or publish packages.

#### Scenario: Successful candidate build
- **WHEN** a maintainer invokes the release-preparation workflow on a supported source revision
- **THEN** the workflow produces temporary reviewable bundles and checksum files without creating a durable GitHub Release or package publication

#### Scenario: Failed candidate verification
- **WHEN** a required target build, checksum, contents check, or executable smoke check fails
- **THEN** the workflow reports failure and does not present that candidate as release-ready

### Requirement: Explicit publication-readiness review

The repository SHALL document review of reachable Git history, GitHub Actions logs/artifacts, and other exposed material for sensitive or non-redistributable content. Publication SHALL be blocked until an owner-approved project license and bundled-content rights are resolved, with consistent manifest claims and required attribution. Missing review evidence, unresolved rights, and sensitive findings MUST remain blockers. Fixture checks alone SHALL NOT satisfy full-history review.

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
