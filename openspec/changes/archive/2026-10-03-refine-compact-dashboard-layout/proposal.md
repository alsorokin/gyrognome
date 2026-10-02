# Proposal

## Why

Make the compact dashboard easier to scan without changing its controls or data. Completed quest history currently consumes space, task progress is mixed into the percentage summary, and the equipment and spell sections lack visual separation.

## What Changes

- Show only `Current quest: <current quest>` in compact mode, not the full quest list.
- Keep XP, encumbrance, plot, and quest percentages together; move task progress to its own row immediately above them using the same progress-bar appearance as full mode.
- Add an empty line before and after compact equipment and after the spellbook.
- Preserve compact scrolling, task prediction, update highlighting, and full-mode presentation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `terminal-dashboard`: Define compact current-quest presentation, a dedicated task progress bar, and section spacing.

## Impact

The change affects `src/dashboard.rs`, its existing Ratatui rendering and scrolling tests, and the compact-layout description in `README.md`. No new dependencies, CLI options, persistence changes, simulation changes, or network behavior are needed. Existing staged dashboard and specification edits are the baseline and must be preserved.
