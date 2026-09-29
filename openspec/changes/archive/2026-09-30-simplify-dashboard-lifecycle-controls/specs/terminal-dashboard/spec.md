# Spec Delta

## MODIFIED Requirements

### Requirement: Live persisted-state refresh

The dashboard SHALL refresh its displayed managed-character state, online
profile metadata, and runtime status at bounded periodic intervals without
requiring a manual refresh action. It MAY read persisted character/profile
state and runtime service status on separate bounded schedules.

When a predicted task position reaches the current task's duration, the
dashboard SHALL wait a bounded settling interval before reading persisted
character state, so that the local runtime has an opportunity to commit the
completed task first. It SHALL separate successive reads triggered this way by
at least that settling interval, and SHALL NOT delay such a read beyond its
regular state-refresh interval.

Except when executing an explicit motto or guild action, the dashboard SHALL
only read persisted character/profile state and service status; it SHALL NOT
own the simulation lock, advance simulation, or write canonical simulation
state. Explicit profile actions MAY write profile metadata through the
online-profile interface while leaving simulation ownership unchanged.

#### Scenario: Viewing an active runtime

- **WHEN** the character's local runtime persists new state while the dashboard
  is open
- **THEN** the dashboard displays the newer persisted state on a subsequent
  refresh without advancing the character itself

#### Scenario: Refreshing profile metadata

- **WHEN** a motto or guild value changes while the dashboard is open
- **THEN** the dashboard displays the newer persisted profile value on a
  subsequent refresh

#### Scenario: Refreshing an inactive runtime

- **WHEN** the character runtime remains inactive while the dashboard is open
- **THEN** the dashboard continues to display the last persisted state and
  updates the displayed inactive or failed service status automatically

#### Scenario: Reading state and service status on separate schedules

- **WHEN** the dashboard reads persisted character and profile state on a
  shorter schedule than runtime service status
- **THEN** it continues to display the most recently read service status and
  does not query the service manager on the shorter schedule

#### Scenario: Reading state after a predicted task ends

- **WHEN** a predicted task position reaches the current task's duration
- **THEN** the dashboard waits a bounded settling interval, giving the local
  runtime an opportunity to commit the completed task, and then reads
  persisted character state without waiting for its next regularly scheduled
  read

#### Scenario: Observing the next task at its starting position

- **WHEN** a read triggered by a saturated prediction returns a newer task than
  the one the prediction was anchored to
- **THEN** the dashboard displays that task's persisted position, rewards, and
  activity description, and anchors its next prediction to them

#### Scenario: Waiting for a runtime that has not yet persisted a completion

- **WHEN** a read triggered by a saturated prediction returns the same task the
  prediction was already anchored to
- **THEN** the dashboard continues to display a full task bar and separates its
  next triggered read from the previous one by at least the settling interval

### Requirement: Lifecycle controls

The dashboard SHALL expose one contextual keyboard action through the existing
user-service lifecycle interface. The action SHALL start an inactive selected
character runtime, stop an active runtime, and recover a failed runtime by
clearing its failed state before starting it. The dashboard SHALL NOT expose a
separate recovery action. Before executing a lifecycle action, it SHALL request
confirmation. It SHALL report successful actions and actionable
service-manager or runtime failures in the dashboard without exiting.

#### Scenario: Starting an inactive runtime

- **WHEN** a user confirms the dashboard lifecycle action for an inactive
  character
- **THEN** the dashboard starts the character through the local user-service
  lifecycle interface and refreshes the displayed status

#### Scenario: Stopping an active runtime

- **WHEN** a user confirms the same dashboard lifecycle action for an active
  character
- **THEN** the dashboard stops the character through the local user-service
  lifecycle interface and refreshes the displayed status

#### Scenario: Recovering a failed runtime

- **WHEN** a user confirms the same dashboard lifecycle action for a failed
  character
- **THEN** the dashboard clears the service's failed state, starts the
  character through the local user-service lifecycle interface, and refreshes
  the displayed status

#### Scenario: Handling a lifecycle failure

- **WHEN** a confirmed lifecycle action fails because the service manager is
  unavailable or the runtime cannot start
- **THEN** the dashboard displays the error and preserves the most recently
  displayed credential-safe character state
