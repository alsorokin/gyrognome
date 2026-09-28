# Third-party material

Gyrognome-authored code is licensed under [MIT](LICENSE). That license does not
replace the rights or notices of upstream works incorporated into the project.

- Desktop Progress Quest 6.4.4-derived tables in `src/desktop_rules.json` and
  related source-derived material reference the author's `v6.4.4` source,
  commit `95f5d66f97a3697a7446fba951ab04156c752274`. The upstream
  `dist/license.txt` notice is preserved in
  [licenses/ProgressQuest-Desktop.txt](licenses/ProgressQuest-Desktop.txt).
  Source: https://bitbucket.org/grumdrig/pq/raw/95f5d66f97a3697a7446fba951ab04156c752274/dist/license.txt
- Browser rules in `src/ruleset.rs` reproduce data from the official
  https://progressquest.com/play/config.js. The official site hosts a
  redistribution notice at https://progressquest.com/license.txt, preserved in
  [licenses/ProgressQuest-Site.txt](licenses/ProgressQuest-Site.txt). Its
  [FAQ](https://progressquest.com/faq.php) explicitly points to the source and
  invites ports; the project owner chose to rely on those official statements
  as the basis for porting the browser data. This documents that assessment,
  not a separate authorization from the original author for every browser
  revision or other third-party material.
- The README screenshot is owner-created and owner-reviewed for privacy.
  Browser/desktop conformance evidence records synthetic and source-derived
  observations; no real saves or signed requests should be distributed.

Neither upstream notice alone clears the repository for publication; see
[release readiness](docs/release-readiness.md) for the remaining privacy and
artifact checks.

No Progress Quest trademark endorsement or affiliation is implied.
