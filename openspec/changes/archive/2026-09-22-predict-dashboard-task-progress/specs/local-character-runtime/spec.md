# Spec Delta

## MODIFIED Requirements

### Requirement: Controlled offline advancement

The system SHALL advance a running managed character using elapsed time
measured only by its active local runtime process and SHALL durably record each
successful resulting state. It SHALL invoke the deterministic simulation
contract with explicit elapsed durations measured from a monotonic clock. For
each scheduled update, it SHALL cap contributed elapsed time at the configured
worker interval and discard excess scheduler-delay time without applying it
later. Time while the runtime is stopped SHALL NOT be applied as catch-up
advancement.

The runtime SHALL schedule each update to occur at the earlier of its
configured worker interval and the completion of the character's active task,
so that a completed task is durably recorded when it completes rather than at
the end of the enclosing interval. Each such advancement duration SHALL be a
whole multiple of the deterministic simulation's bounded tick duration and
SHALL be at least one such tick, so that the sequence of ticks supplied to the
simulation, and therefore the resulting canonical state and random
continuation, are unchanged by this scheduling. The configured worker interval
SHALL remain an upper bound on any single advancement, and this scheduling
SHALL NOT increase the total simulated time advanced per unit of real time.

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

#### Scenario: Recording a task completion when it occurs

- **WHEN** the active task would complete before the configured worker interval
  elapses
- **THEN** the runtime advances only as far as that task's completion and
  durably records the resulting state, so the next persisted state reflects the
  following task rather than the completed one

#### Scenario: Advancing a task longer than the worker interval

- **WHEN** the active task would not complete within the configured worker
  interval
- **THEN** the runtime advances by one configured interval and leaves the task
  active

#### Scenario: Preserving progression under boundary-aligned scheduling

- **WHEN** a runtime advances a character across one or more task completions
  using task-aligned scheduling, and the same starting state is advanced across
  the same total duration using only whole worker intervals
- **THEN** both produce the same canonical state and random continuation

#### Scenario: Active task with no remaining time

- **WHEN** the active task is already complete or its remaining time is shorter
  than the simulation's bounded tick duration
- **THEN** the runtime still advances by at least one whole tick rather than
  scheduling a zero-duration update
