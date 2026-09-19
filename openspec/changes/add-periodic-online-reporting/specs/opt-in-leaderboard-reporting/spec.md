# Spec Delta

## MODIFIED Requirements

### Requirement: No implicit or automated reporting

The system SHALL submit browser-compatible reports only for persisted
level-up and act-completion events from an active online managed-character
runtime, or for an operator's explicit manual-brag action. Automatic reports
SHALL be one-attempt best-effort deliveries and SHALL not be queued or retried.
Registration, inspection, dashboard refresh, lifecycle operations, and
administration SHALL NOT independently submit leaderboard reports.

#### Scenario: Advancing a managed online character

- **WHEN** an active managed online character persists an advancement that
  emits a level-up or act-completion report event
- **THEN** the runtime submits exactly one corresponding report without
  requiring foreground confirmation

#### Scenario: Advancing a managed character

- **WHEN** a managed character advances through its worker or local lifecycle
- **THEN** the system submits reports only for persisted level-up and
  act-completion events from an online worker

#### Scenario: Advancing a managed offline character

- **WHEN** an active managed character without an online credential advances
- **THEN** the runtime submits no leaderboard request

#### Scenario: Refreshing a dashboard

- **WHEN** the dashboard refreshes a managed character without the operator
  invoking its manual-brag action
- **THEN** it sends no leaderboard request
