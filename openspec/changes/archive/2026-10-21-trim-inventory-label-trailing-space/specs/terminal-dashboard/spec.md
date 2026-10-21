# Spec Delta

## ADDED Requirements

### Requirement: Clean inventory label spacing

The dashboard SHALL omit trailing literal spaces from inventory names when rendering inventory labels. It SHALL preserve the original inventory names in stored state and compatibility data.

#### Scenario: Rendering an inventory name with trailing spaces

- **WHEN** an inventory name ends with one or more literal spaces
- **THEN** the dashboard displays the name without those trailing spaces before its quantity
- **AND** the inventory name in stored state remains unchanged
