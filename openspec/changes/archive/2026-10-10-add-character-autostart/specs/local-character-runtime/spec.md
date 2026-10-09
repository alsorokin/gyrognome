# Spec Delta

## ADDED Requirements

### Requirement: Persistent per-character autostart

The system SHALL provide `gyro autostart <id> on|off` to enable or disable
persistent automatic startup of a registered character's user service.
Autostart SHALL remain independent of current service activity: changing it
SHALL NOT start, stop, or recover a worker, and Start/Stop/Recover SHALL NOT
change it. Repeating the same setting SHALL succeed without changing runtime
activity. Registration SHALL NOT automatically enable autostart.

Enabled characters SHALL start when the user's service manager starts,
including after reboot when that manager starts. The system SHALL NOT promise
startup before login unless account-wide lingering is configured. It SHALL
document that prerequisite without automatically enabling lingering or
requesting elevated privileges. Automatic startup SHALL retain existing
persisted-state, exclusive-ownership, reporting-eligibility, and rested-time
semantics, including no immediate downtime catch-up.

#### Scenario: Enabling an inactive character

- **WHEN** a user enables autostart for a registered inactive character
- **THEN** automatic startup is persistently enabled and the worker remains inactive

#### Scenario: Disabling an active character

- **WHEN** a user disables autostart for an active character
- **THEN** automatic startup is disabled and the worker remains active

#### Scenario: Returning after a reboot

- **WHEN** the user's service manager starts after reboot and a character has autostart enabled
- **THEN** it launches that character without opening the dashboard or manually starting it
- **AND** the worker resumes its persisted state without instant downtime advancement

#### Scenario: Stopping an enabled character

- **WHEN** a user stops a character with autostart enabled
- **THEN** it stops for the current session and retains automatic startup for the next user-manager startup

#### Scenario: Repeating a preference

- **WHEN** a user sets autostart to its already configured persistent value
- **THEN** the command succeeds without starting or stopping the worker

#### Scenario: Invalid character or unsuccessful configuration

- **WHEN** the identifier is invalid or unknown, the service template is unavailable, or automatic startup cannot be configured
- **THEN** the command reports an actionable error without claiming success or changing character data

### Requirement: Observable autostart configuration

`gyro status` SHALL report autostart separately from service activity and runtime
ownership in text and JSON. Its `autostart` JSON field SHALL distinguish
`enabled`, `disabled`, and `unavailable`. Enabled SHALL mean persistent startup,
not temporary runtime-only enablement. Unavailable or unsupported configuration
SHALL include an actionable diagnostic rather than being presented as disabled.
Status inspection SHALL NOT change enablement, character state, or account-wide
configuration.

#### Scenario: Enabled but stopped

- **WHEN** status is requested for an inactive character with persistent autostart enabled
- **THEN** output reports enabled autostart separately from inactive service activity

#### Scenario: External preference change

- **WHEN** persistent service enablement is changed outside Gyrognome
- **THEN** subsequent status reports reflect the service-manager configuration

#### Scenario: Unknown startup configuration

- **WHEN** startup configuration cannot be determined
- **THEN** status identifies autostart as unavailable with a diagnostic rather than asserting it is off
