# Design

## Context

See proposal.md for motivation. The manual report is exposed through both `gyro report` and the dashboard Brag action, while worker progress reports run independently. The project targets Linux and already has the `url` crate for URL construction.

## Goals / Non-Goals

**Goals:**

- Open the public, realm-specific character page after an explicit manual-brag attempt from either UI.
- Keep browser launch failures separate from the report delivery result.
- Use only a fixed official public-page URL and safe character identity data.

**Non-Goals:**

- Opening pages after worker-generated level/act reports or motto/guild changes.
- Polling the leaderboard or claiming that opening the page proves the report was accepted.
- Adding cross-platform browser-launch support beyond the Linux-native project environment.

## Decisions

- **Construct a public URL, not a report URL.** Map the character's supported realm to its fixed official public leaderboard page and add the display name as an encoded `name` query value. Do not reuse a saved endpoint or report request, which could contain private authentication data.
- **Open only after an explicit manual attempt.** Trigger the opener after the one report call returns an outcome, including rejection or delivery failure. A declined confirmation, an eligibility failure before submission, and automatic reporting do not open anything.
- **Use the system browser launcher without a new dependency.** Invoke Linux `xdg-open` with the completed HTTPS URL. Report launch failure independently so it cannot replace or obscure the report's categorized outcome.
- **Share URL/opening behavior across the CLI and dashboard.** Keep the two manual-brag paths consistent, and cover name encoding, realm routing, and launch failure through focused tests.

## Risks / Trade-offs

- [No graphical browser is available in a terminal session] → Report the browser-opening failure while preserving the completed report outcome.
- [A future supported realm lacks a public-page mapping] → Refuse to construct a page URL for an unknown realm rather than falling back to another realm or a saved endpoint.

## Migration Plan

No data migration is needed. Update the README's manual-report description; rollback consists of removing the opener calls and helper.
