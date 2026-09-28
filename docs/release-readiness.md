# Private release readiness checklist

**Default: BLOCKED.** Candidate archives are not public downloads. Do not
change repository visibility, upload a GitHub Release, or publish to any
registry by following this checklist alone. An owner must explicitly authorize
each later publication action.

- [x] Inspect **all Git refs**, including branches, tags and stash, and
  reachable history for secrets, player saves, browser profiles, raw requests,
  internal URLs, and material not approved for redistribution. Fixture-safety
  tests cover only current test fixtures, not history. The private review
  records heuristic classifications and the owner's acceptance of old,
  directly addressable commit identities; this is not proof of erasure.
- [x] Review tracked source/assets plus ignored local inputs and any release
  archive against an explicit file allowlist. Never upload a checkout, local
  `.mcp.json`, `saves/`, `target/`, `.playwright-mcp/`, or private review notes.
  Ignored private inputs were inventoried and excluded without opening them;
  local trial archives have only allowlisted files. Clean CI candidates are
  still required after the notice changes.
- [ ] Review GitHub issues, PRs and their attachments, Actions runs/logs and
  artifacts, releases/assets, wiki/Pages if enabled, and the README screenshot.
  The owner created and privacy-reviewed the screenshot, but its link returned
  404 to an anonymous HEAD request; fix or remove a broken reference before
  public launch.
  Recheck immediately before changing visibility; Actions history and logs
  become visible when a private repository becomes public.
- [x] Privately disposition the credential-like and signed-query-like history
  matches as parser/test examples; rotate affected credentials first if any
  future review identifies a real one. The owner authorized rewriting the
  corporate email/name in both branches and accepted residual SHA-link access
  to an old commit. Any later remediation to history or GitHub artifacts
  requires a separate reviewed action and a fresh audit.
- [x] Obtain the owner's **explicit license choice** for Gyrognome-authored
  code, add its notice, and align Cargo/npm metadata. The owner chose MIT with
  "Gyrognome contributors" as the holder. This does **not** clear publication:
  separately document the owner's assessment of rights and attribution for the
  browser `config.js` snapshot (`src/ruleset.rs`), desktop-derived data
  (`src/desktop_rules.json`), conformance assets, and README image. The owner
  relies on the official site's license and FAQ invitation to port; see
  [third-party notices](../THIRD_PARTY_NOTICES.md). This is not an independent
  legal determination.
  A fork's license does not establish rights to another upstream version.
- [ ] Inspect both native Linux candidate builds, SHA-256 checks, architecture
  and measured GLIBC floor, smoke checks, and user-local install/service and
  backup/rollback exercises. Record platform limitations accurately: the ARM
  install trial used emulation and native ARM user-service lifecycle has not
  been exercised.
- [x] Record approvals and blockers in a **private**, redacted review record
  outside version control. Never put credential values into a review artifact.

If any item is unchecked or unresolved, status stays **BLOCKED**. The current
private review record is `target/release-publication-review.md` (ignored by
Git); it records the classified credential-pattern matches, remaining
redistribution/image questions and verification status. That record is not a
substitute for a complete manual review. Running the manually triggered
GitHub Actions preparation workflow produces private, short-lived candidate
artifacts only; do not treat it as publication approval.
