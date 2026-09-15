## MODIFIED Requirements

### Requirement: Controlled offline advancement

The system SHALL advance a running managed character using elapsed time measured
only by its active local runtime process and SHALL durably record each successful
resulting state. It SHALL invoke the deterministic simulation contract with
explicit elapsed durations measured from a monotonic clock. For each scheduled
update, it SHALL cap contributed elapsed time at the configured worker interval
and discard excess scheduler-delay time without applying it later. Time while the
runtime is stopped SHALL NOT be applied as catch-up advancement.

#### Scenario: Advancing while the runtime is active

- **WHEN** a managed character runtime remains active across one or more
  advancement intervals
- **THEN** the system persists each resulting canonical state and exposes the
  accumulated progression through local inspection

#### Scenario: Restarting a stopped runtime

- **WHEN** a managed character runtime is stopped and later started
- **THEN** the system resumes from the most recently persisted state without
  applying elapsed downtime

#### Scenario: Simulation cannot advance a state

- **WHEN** deterministic simulation reports an unsupported transition or other
  error
- **THEN** the runtime reports the failure, retains the last successful state,
  and does not fabricate an advancement result

#### Scenario: Delayed scheduled update

- **WHEN** a worker callback occurs later than its configured interval because
  of scheduler delay or machine suspension
- **THEN** the worker advances by no more than one configured interval and does
  not apply the discarded excess on a later callback

#### Scenario: Unrepresentable elapsed duration

- **WHEN** an explicit worker advancement duration cannot be represented in
  milliseconds
- **THEN** the runtime reports an elapsed-duration error and retains the last
  successfully persisted state
