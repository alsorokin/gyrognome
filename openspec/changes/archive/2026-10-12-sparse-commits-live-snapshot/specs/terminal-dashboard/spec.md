# Spec Delta

## MODIFIED Requirements

### Requirement: Live persisted-state refresh

The dashboard SHALL refresh its displayed managed-character state, online
profile metadata, and runtime status at bounded periodic intervals without
requiring a manual refresh action. It MAY read persisted character/profile state
and runtime service status on separate bounded schedules.

While a live local runtime owns the character, the dashboard SHALL read
character state and rested accounting from that runtime's published live
state instead of the managed-character database. Otherwise, it SHALL read the
last durably persisted state. Online profile metadata, compatibility, and
service status SHALL continue to come from their persisted sources. Everywhere
in this specification, "persisted state" read by the dashboard includes such
live runtime state.

When a predicted task position reaches the current task's duration, the
dashboard SHALL wait a bounded settling interval before reading persisted
character state, so that the local runtime has an opportunity to publish the
completed task first. It SHALL separate successive reads triggered this way by
at least that settling interval, and SHALL NOT delay such a read beyond its
regular state-refresh interval.

Except when executing an explicit motto or guild action, the dashboard SHALL
only read persisted character/profile state, live runtime state, and service
status; it SHALL NOT own the simulation lock, advance simulation, or write
canonical simulation state or live runtime state. Explicit profile actions MAY
write profile metadata through the online-profile interface while leaving
simulation ownership unchanged.

#### Scenario: Viewing an active runtime

- **WHEN** the character's local runtime publishes new live state while the
  dashboard is open
- **THEN** the dashboard displays the newer state on a subsequent refresh
  without advancing the character itself, even if the runtime has not yet
  durably committed it

#### Scenario: Ignoring live state from a runtime that no longer owns the character

- **WHEN** live state exists but its publishing process no longer owns the
  character
- **THEN** the dashboard displays the last durably persisted state

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
  runtime an opportunity to publish the completed task, and then reads
  character state without waiting for its next regularly scheduled read

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
