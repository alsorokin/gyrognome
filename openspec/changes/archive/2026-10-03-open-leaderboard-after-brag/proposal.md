# Proposal

## Why

After manually bragging, operators currently have no direct way to see the character's public leaderboard entry. Open the matching Progress Quest leaderboard page so the result can be checked immediately.

## What Changes

- After an eligible manual brag is attempted through `gyro report` or the dashboard Brag action, open the character's public leaderboard page with its display name selected.
- Build the page from the character's supported realm and safely encode its name; do not include credentials or report-request data.
- Keep automatic level/act reports and other profile actions from opening a browser. Report delivery outcomes remain independent of whether the public page opens.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `opt-in-leaderboard-reporting`: Explicit manual brag actions also open the matching public leaderboard page.

## Impact

The CLI manual-report flow and dashboard Brag action will invoke the system browser for a fixed official Progress Quest page. The existing leaderboard-reporting contract and README usage documentation will be updated; no new dependency or server request is needed.
