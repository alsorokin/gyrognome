# Spec Delta

## ADDED Requirements

### Requirement: Open the public leaderboard after manual brag

After an eligible manual brag has been explicitly confirmed and its single
report attempt has completed, the system SHALL open the selected character's
public Progress Quest leaderboard page through the system browser. The page
SHALL correspond to the character's supported realm and identify the character
by its display name, safely encoded as a query value. The page URL SHALL use a
fixed official HTTPS origin and SHALL NOT contain credentials, signed report
data, or user-supplied endpoint data. This behavior SHALL apply to both the
confirmed CLI report command and the dashboard Brag action, regardless of the
categorized report-delivery outcome. Failure to open the page SHALL be reported
separately and SHALL NOT change the report outcome.

#### Scenario: Opening the leaderboard after a confirmed CLI brag

- **WHEN** an operator confirms an eligible character's manual brag from the CLI and its one report attempt completes
- **THEN** the system opens that character's realm-specific public leaderboard page with the display name safely encoded

#### Scenario: Opening the leaderboard after a dashboard brag

- **WHEN** an operator invokes an eligible character's Brag action in the dashboard and its one report attempt completes
- **THEN** the system opens that character's realm-specific public leaderboard page with the display name safely encoded

#### Scenario: Report delivery is rejected or fails

- **WHEN** the confirmed manual-brag report attempt completes with a rejected or failed delivery outcome
- **THEN** the public leaderboard page is still opened and the delivery outcome remains unchanged

#### Scenario: No confirmed manual brag occurs

- **WHEN** an operator declines the CLI confirmation, a manual action is blocked before submission, or a worker sends an automatic progress report
- **THEN** no public leaderboard page is opened

#### Scenario: The system browser cannot be opened

- **WHEN** the public leaderboard page cannot be opened through the system browser
- **THEN** the report outcome is preserved and a separate safe opening failure is reported
