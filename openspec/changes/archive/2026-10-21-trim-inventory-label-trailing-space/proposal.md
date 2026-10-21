# Proposal

## Why

Classic-derived inventory names can include trailing spaces, which become visible before the quantity in the dashboard. Keep the source name intact for compatibility while making inventory labels read cleanly.

## What Changes

- Trim trailing spaces from inventory names when rendering dashboard labels.
- Preserve the stored inventory name and all simulation, save, and compatibility behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `terminal-dashboard`: Inventory labels omit trailing spaces in dashboard output.

## Impact

The shared dashboard inventory formatter and its focused rendering tests; no data, save format, or compatibility-table changes.
