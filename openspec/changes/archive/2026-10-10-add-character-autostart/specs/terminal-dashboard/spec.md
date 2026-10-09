# Spec Delta

## ADDED Requirements

### Requirement: Dashboard autostart preference

The dashboard SHALL display `Autostart: On`, `Autostart: Off`, or
`Autostart: Unavailable` in both full and compact layouts, separately from
runtime activity. It SHALL expose `a` as a documented autostart toggle, distinct
from the existing contextual Start/Stop/Recover action. Before changing the
preference it SHALL request confirmation naming the target setting and
explaining that it does not start or stop the current worker. Enter SHALL
confirm and Escape SHALL cancel without configuration changes.

The dashboard SHALL use the same autostart interface as the CLI, refresh the
preference on its bounded service-status schedule and after successful
configuration, and reflect external enablement changes. It SHALL NOT guess a
target setting when autostart is unavailable; it SHALL show the diagnostic
instead. Action outcomes SHALL remain visible for at least five seconds,
following existing action-message behavior. Errors SHALL preserve displayed
character state and keep the dashboard open. Autostart actions SHALL NOT
advance simulation, modify profile metadata, or send leaderboard requests.

#### Scenario: Discovering autostart in either layout

- **WHEN** a user views an enabled character in a full or compact dashboard
- **THEN** it shows `Autostart: On` independently of current service activity and documents the `a` shortcut

#### Scenario: Confirming an enable action

- **WHEN** a user presses `a` for an autostart-disabled inactive character and confirms with Enter
- **THEN** the dashboard enables autostart, refreshes its preference, reports the result, and leaves the worker inactive

#### Scenario: Cancelling a toggle

- **WHEN** a user presses Escape at an autostart confirmation
- **THEN** the dashboard leaves both autostart configuration and runtime activity unchanged

#### Scenario: Disabling startup without stopping

- **WHEN** a user confirms the autostart toggle for an enabled active character
- **THEN** the dashboard disables autostart without stopping the worker

#### Scenario: Configuration unavailable or action fails

- **WHEN** autostart state cannot be determined or a confirmed configuration action fails
- **THEN** the dashboard displays an actionable error without guessing the preference, changing displayed character state, or exiting

#### Scenario: Observing an external enablement change

- **WHEN** autostart is changed through the CLI or service-manager tools while the dashboard is open
- **THEN** the dashboard displays the new preference on a subsequent service-status refresh
