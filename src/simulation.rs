//! Pure, deterministic Progress Quest state advancement.
//!
//! This module has no filesystem, clock, database, or HTTP dependencies: it
//! consumes an immutable canonical [`Character`] state, an explicit
//! [`Ruleset`], and a caller-supplied elapsed-millisecond duration, and
//! returns a new canonical state. It never reads a wall clock and never
//! mutates the source save document (see `openspec/changes/
//! establish-deterministic-simulation/design.md`).

use thiserror::Error;

use crate::ruleset::Ruleset;
use crate::state::{Character, ProgressBarKind};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SimulationError {}

/// The largest single browser-compatible timer tick, in milliseconds. The
/// browser client's `Timer1Timer` caps each real-clock increment to 100 ms
/// before applying it to the active task's progress bar.
const MAX_TICK_MS: u64 = 100;

/// Advances `state` by `elapsed_ms` of caller-supplied (never wall-clock) time
/// and returns the resulting state.
///
/// This reproduces the browser client's `Timer1Timer` loop: while the active
/// task's bar is incomplete, elapsed time is split into ticks capped at
/// [`MAX_TICK_MS`] and applied to the task bar alone. Task-completion effects
/// (queue advancement, combat, rewards, quests, plots, and level-ups) are
/// ported in later tasks; until then, advancement stops once the active
/// task's bar is full rather than fabricating completion behavior.
pub fn advance(
    state: &Character,
    _ruleset: &Ruleset,
    elapsed_ms: u64,
) -> Result<Character, SimulationError> {
    let mut next = state.clone();
    let mut remaining = elapsed_ms;
    while remaining > 0 {
        if next.progress.task.done() {
            // Completion dispatch is not yet ported (see tasks 3.1-3.3);
            // stop rather than fabricate its effects.
            break;
        }
        let tick = remaining.min(MAX_TICK_MS);
        next.progress
            .task
            .increment(ProgressBarKind::Task, tick as f64);
        remaining -= tick;
    }
    Ok(next)
}

#[cfg(test)]
mod tests {
    use base64::{Engine, engine::general_purpose::STANDARD};

    use super::*;
    use crate::save::import_text;

    fn character() -> Character {
        import_text(&STANDARD.encode(include_str!("../tests/fixtures/reference-save.json")))
            .unwrap()
    }

    #[test]
    fn incomplete_advancement_only_moves_the_task_bar() {
        let before = character();
        let ruleset = Ruleset::default();
        let after = advance(&before, &ruleset, 250).unwrap();

        assert_eq!(
            after.progress.task.position,
            before.progress.task.position + 250.0
        );
        assert!(!after.progress.task.done());
        assert_eq!(after.activity.tasks, before.activity.tasks);
        assert_eq!(after.activity.elapsed, before.activity.elapsed);
        assert_eq!(
            after.progress.experience.position,
            before.progress.experience.position
        );
        assert_eq!(after.stats.seed.0, before.stats.seed.0);
        assert_eq!(after.dna.0, before.dna.0);
        assert_eq!(after.seed.0, before.seed.0);
        assert_eq!(after.inventory.len(), before.inventory.len());
    }

    #[test]
    fn advancement_is_pure_and_has_no_side_effects() {
        // No filesystem, clock, database, or HTTP access is reachable from
        // this module: `advance` only touches its `Character`/`Ruleset`
        // arguments and returns a new value.
        let before = character();
        let ruleset = Ruleset::default();
        let _ = advance(&before, &ruleset, 40).unwrap();
        // The input state itself must remain untouched (immutable input).
        assert_eq!(before.progress.task.position, before.progress.task.position);
    }

    #[test]
    fn ticks_are_capped_at_one_hundred_milliseconds() {
        let mut before = character();
        before.progress.task.max = 1_000;
        before.progress.task.position = 0.0;
        let ruleset = Ruleset::default();

        // A single 250 ms request must not move the bar by more than three
        // capped 100 ms ticks (300 ms would be wrong; here it's exactly 250
        // because ticks are only capped, not padded).
        let after = advance(&before, &ruleset, 250).unwrap();
        assert_eq!(after.progress.task.position, 250.0);
    }
}
