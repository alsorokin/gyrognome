## MODIFIED Requirements

### Requirement: Credential-safe local-only presentation

The dashboard SHALL never display passkeys, account logins/passwords, raw save
documents/properties, signed/authenticated endpoint URLs, raw responses, or
unrecognized raw fields. It SHALL provide immediate manual brag and editable
motto/guild actions for an eligible character through its approved official
endpoint. Editors SHALL show current persisted values, accept printable
non-control Unicode input, use Enter to submit, and Escape to cancel. Profile
encoding restrictions SHALL be enforced before persistence or transport.
Empty guild input SHALL request leaving; no separate leave action is needed.
Only safe categorized outcomes SHALL be shown. Refresh, navigation, rendering,
and lifecycle actions SHALL NOT independently issue HTTP requests.

#### Scenario: Bragging from the dashboard

- **WHEN** an operator invokes manual brag for an eligible inactive character
- **THEN** one profile-compatible manual report is immediately attempted and a
  safe delivery outcome is displayed

#### Scenario: Changing a motto from the dashboard

- **WHEN** an eligible character's valid motto edit is submitted or cleared
- **THEN** one motto action is performed, the displayed profile is refreshed,
  and a safe outcome is shown

#### Scenario: Submitting a guild designation from the dashboard

- **WHEN** valid empty or non-empty guild input is submitted for an eligible
  character
- **THEN** one guild action is performed, the displayed profile is refreshed,
  and a safe category is shown

#### Scenario: Cancelling a profile editor

- **WHEN** an operator presses Escape in a motto or guild editor
- **THEN** it closes without state changes or requests

#### Scenario: Viewing an online-originated character

- **WHEN** a character imported from an online browser or desktop save is opened
- **THEN** only safe canonical/profile state and local status are shown,
  without credentials or any request until an explicit online action

#### Scenario: Refreshing without bragging

- **WHEN** the dashboard refreshes, renders, navigates, or performs a lifecycle
  action without an explicit online action
- **THEN** it sends no leaderboard request

## ADDED Requirements

### Requirement: Visible compatibility and eligibility

The dashboard SHALL display the persisted continuation profile and safe
online-operation eligibility, including the reason for classic conformance
gating or local-only provenance and the fresh-import recovery instruction.
It SHALL distinguish unavailable desktop history from counters measured since
import. Profile selection SHALL come from persisted state, not display labels.
Task-bar prediction SHALL NOT dispatch completion, create report eligibility,
or replace a pending desktop full-bar state with an assumed next task.

#### Scenario: Viewing a desktop local-only fork

- **WHEN** an online-originated desktop import has advanced while reporting
  was gated
- **THEN** the dashboard shows local-only status, disables online actions, and
  explains that later online use requires a fresh official-client import

#### Scenario: Rendering a pending desktop completion

- **WHEN** a desktop task is full but its completion callback is not persisted
- **THEN** the display holds the full bar without fabricating rewards, task
  counts, next activity, or reports
