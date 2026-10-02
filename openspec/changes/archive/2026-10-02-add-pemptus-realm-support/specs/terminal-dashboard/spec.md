# Spec Delta

## ADDED Requirements

### Requirement: Partial desktop operation availability

The dashboard SHALL distinguish fully eligible, wholly ineligible, and partially
eligible desktop characters. When some but not all operations are eligible, it
SHALL show
which automatic level, automatic act, manual brag, motto, and guild operations
are available, with safe reasons for gated operations. Brag and profile editors
SHALL be gated by their requested operation rather than an all-operations
summary.

Local-only provenance SHALL continue to disable every online action and show
the fresh-official-import recovery instruction. Presentation SHALL NOT itself
send requests, change provenance, or confer eligibility. Browser presentation
and the concise uniform-state desktop summary SHALL remain unchanged.

#### Scenario: Showing manual-only availability

- **WHEN** a fresh desktop managed import has eligible manual brag but gated
  automatic, motto, and guild operations
- **THEN** the dashboard identifies partial availability, exposes manual brag,
  and explains the unavailable operations without labeling all actions blocked

#### Scenario: Opening a supported motto editor with guild gated

- **WHEN** motto is eligible and guild is ineligible for a Pemptus character
- **THEN** the existing motto editor remains usable while the guild action
  reports its own safe blocking reason

#### Scenario: Showing uniform eligibility

- **WHEN** every desktop online operation has the same eligibility outcome
- **THEN** the dashboard retains a concise overall summary without unnecessary
  mixed-operation detail

#### Scenario: Showing usable supported Pemptus operations

- **WHEN** an otherwise eligible unadapted, spelling-only, or placeholder-only
  Pemptus import has valid reporting/motto coverage and public-confirmation guild readiness
- **THEN** all five operations are available with existing controls and no
  obsolete import-path mismatch or automatic-fork warning

#### Scenario: Showing a local-only Pemptus timeline

- **WHEN** a Pemptus managed timeline has durable local-only provenance
- **THEN** every online action is disabled and the dashboard instructs the
  operator to use a fresh official-client import

#### Scenario: Refreshing partial availability

- **WHEN** the dashboard renders or refreshes eligibility details
- **THEN** it sends no leaderboard request and does not alter character state
