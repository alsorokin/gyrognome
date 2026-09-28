# Private release readiness checklist

**Default: BLOCKED.** Candidate archives are not public downloads. Do not
change repository visibility, upload a GitHub Release, or publish to any
registry by following this checklist alone. An owner must explicitly authorize
each later publication action.

- [ ] Inspect **all Git refs**, including branches, tags and stash, and
  reachable history for secrets, player saves, browser profiles, raw requests,
  internal URLs, and material not approved for redistribution. Fixture-safety
  tests cover only current test fixtures, not history.
- [ ] Review tracked source/assets plus ignored local inputs and any release
  archive against an explicit file allowlist. Never upload a checkout, local
  `.mcp.json`, `saves/`, `target/`, `.playwright-mcp/`, or private review notes.
- [ ] Review GitHub issues, PRs and their attachments, Actions runs/logs and
  artifacts, releases/assets, wiki/Pages if enabled, and the README screenshot.
  Recheck immediately before changing visibility; Actions history and logs
  become visible when a private repository becomes public.
- [ ] Privately disposition every sensitive-content candidate; rotate affected
  credentials first if anything was exposed. Remediation to history or GitHub
  artifacts requires a separate reviewed action and a fresh audit.
- [ ] Obtain the owner's **explicit license choice** for Gyrognome. Resolve
  rights and attribution separately for the verbatim browser `config.js`
  snapshot (`src/ruleset.rs`), desktop-derived data (`src/desktop_rules.json`),
  conformance assets, and README image. A fork's license does not establish
  rights to another upstream version. Add approved license/notices and keep
  Cargo/npm metadata consistent only after this decision.
- [ ] Inspect both native Linux candidate builds, SHA-256 checks, architecture
  and measured GLIBC floor, smoke checks, and user-local install/service and
  backup/rollback exercises. Record platform limitations accurately: the ARM
  install trial used emulation and native ARM user-service lifecycle has not
  been exercised.
- [ ] Record approvals and blockers in a **private**, redacted review record
  outside version control. Never put credential values into a review artifact.

If any item is unchecked or unresolved, status stays **BLOCKED**. The current
private review record is `target/release-publication-review.md` (ignored by
Git); it records unresolved credential-pattern matches, redistribution terms,
and owner licensing. That record is not a substitute for a complete manual
review. Running the manually triggered GitHub Actions preparation workflow
produces private, short-lived candidate artifacts only; do not treat it as
publication approval.
