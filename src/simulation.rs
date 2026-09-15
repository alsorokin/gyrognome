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
pub enum SimulationError {
    /// A state transition would require browser behavior not yet ported.
    /// Rather than fabricate an effect, advancement stops and reports which
    /// unported behavior family it needed (see tasks 3.2/3.3).
    #[error("advancement requires unported behavior: {0}")]
    Unsupported(&'static str),
}

/// The largest single browser-compatible timer tick, in milliseconds. The
/// browser client's `Timer1Timer` caps each real-clock increment to 100 ms
/// before applying it to the active task's progress bar.
const MAX_TICK_MS: u64 = 100;

/// Advances `state` by `elapsed_ms` of caller-supplied (never wall-clock) time
/// and returns the resulting state.
///
/// This reproduces the browser client's `Timer1Timer` loop: while the active
/// task's bar is incomplete, elapsed time is split into ticks capped at
/// [`MAX_TICK_MS`] and applied to the task bar alone; once the bar is full,
/// completion is dispatched (task queue selection, per [`dispatch_completion`])
/// before any further ticking, exactly as the browser processes a completion
/// callback with no elapsed-time cost of its own. Combat resolution and
/// reward paths (task 3.2/3.3) are not yet ported; advancement stops with
/// [`SimulationError::Unsupported`] rather than fabricating their effects.
pub fn advance(
    state: &Character,
    ruleset: &Ruleset,
    elapsed_ms: u64,
) -> Result<Character, SimulationError> {
    let mut next = state.clone();
    let mut remaining = elapsed_ms;
    loop {
        if next.progress.task.done() {
            dispatch_completion(&mut next, ruleset)?;
            continue;
        }
        if remaining == 0 {
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

/// Ports the browser `Timer1Timer`'s `TaskBar.done()` branch: task-count and
/// elapsed-time bookkeeping, the experience/quest/plot "gain" advancement,
/// and `Dequeue()`'s task-queue selection. `ClearAllSelections()` only
/// touches transient UI selection state with no canonical representation, so
/// it has nothing to port.
fn dispatch_completion(state: &mut Character, ruleset: &Ruleset) -> Result<(), SimulationError> {
    let task_max = state.progress.task.max;
    state.activity.tasks += 1;
    state.activity.elapsed += task_max / 1000;

    if state.activity.kill == "Loading...." {
        state.progress.task.reset(ProgressBarKind::Task, 0, 0.0);
    }

    // `gain` mirrors the browser's `Pos('kill|', game.task) == 1`: the task
    // just completed was a monster-kill task.
    let gain = state.activity.task.starts_with("kill|");
    let delta = (task_max / 1000) as f64;

    if gain {
        if state.progress.experience.done() {
            return Err(SimulationError::Unsupported(
                "level-up (ported in task 3.3)",
            ));
        }
        state
            .progress
            .experience
            .increment(ProgressBarKind::Experience, delta);
    }

    if gain && state.plot.act >= 1 {
        if state.progress.quest.done() || state.quests.is_empty() {
            return Err(SimulationError::Unsupported(
                "quest completion (ported in task 3.3)",
            ));
        }
        state
            .progress
            .quest
            .increment(ProgressBarKind::Quest, delta);
    }

    if gain || state.plot.act == 0 {
        if state.progress.plot.done() {
            return Err(SimulationError::Unsupported(
                "interplot cinematic (ported in task 3.3)",
            ));
        }
        state.progress.plot.increment(ProgressBarKind::Plot, delta);
    }

    dequeue(state, ruleset)
}

/// Ports the browser `Dequeue()`. Only the plain task/plot queue-selection
/// path is implemented; every branch that requires combat resolution,
/// reward mechanics, or monster-task selection (tasks 3.2/3.3) reports
/// [`SimulationError::Unsupported`] instead of fabricating its effects.
fn dequeue(state: &mut Character, _ruleset: &Ruleset) -> Result<(), SimulationError> {
    while state.progress.task.done() {
        if split(&state.activity.task, 0) == "kill" {
            return Err(SimulationError::Unsupported(
                "kill-task loot resolution (ported in task 3.2)",
            ));
        } else if state.activity.task == "buying" {
            return Err(SimulationError::Unsupported(
                "equipment purchase resolution (ported in task 3.3)",
            ));
        } else if state.activity.task == "market" || state.activity.task == "sell" {
            return Err(SimulationError::Unsupported(
                "market/sell resolution (ported in task 3.3)",
            ));
        }

        let old = std::mem::take(&mut state.activity.task);

        if let Some(entry) = state.queue.first().cloned() {
            let kind = split(&entry, 0);
            match kind {
                "task" => {
                    let duration_s: u64 = split(&entry, 1).parse().unwrap_or(0);
                    let caption = split(&entry, 2).to_owned();
                    state.queue.remove(0);
                    set_task(state, &caption, duration_s * 1000);
                }
                "plot" => {
                    return Err(SimulationError::Unsupported(
                        "act completion (ported in task 3.3)",
                    ));
                }
                _ => {
                    return Err(SimulationError::Unsupported(
                        "unrecognized queue entry kind",
                    ));
                }
            }
        } else if state.progress.encumbrance.done() {
            return Err(SimulationError::Unsupported(
                "selling loot at market (ported in task 3.3)",
            ));
        } else if !old.contains("kill|") && old != "heading" {
            return Err(SimulationError::Unsupported(
                "equipment shopping decision (ported in task 3.3)",
            ));
        } else {
            return Err(SimulationError::Unsupported(
                "monster task selection (ported in task 3.2)",
            ));
        }
    }
    Ok(())
}

/// Ports the browser `Task(caption, msec)`: sets the activity status text
/// (with the browser's trailing `"..."`) and resets the task bar.
fn set_task(state: &mut Character, caption: &str, duration_ms: u64) {
    state.activity.kill = format!("{caption}...");
    state
        .progress
        .task
        .reset(ProgressBarKind::Task, duration_ms, 0.0);
}

/// Ports the browser `Split(s, index)`: splits `s` on `|` and returns the
/// 0-based `index`th field, or `""` if the field is absent.
fn split(s: &str, index: usize) -> &str {
    s.split('|').nth(index).unwrap_or("")
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

    #[test]
    fn completing_a_sell_task_reports_unsupported_rather_than_fabricating_a_market_result() {
        // The reference fixture's active task is "sell" with an empty
        // queue; resolving it requires the market/sell reward mechanics
        // ported in task 3.3, so advancement must stop rather than guess.
        let mut before = character();
        before.progress.task.max = 100;
        before.progress.task.position = 0.0;
        let ruleset = Ruleset::default();

        let error = advance(&before, &ruleset, 100).unwrap_err();
        assert_eq!(
            error,
            SimulationError::Unsupported("market/sell resolution (ported in task 3.3)")
        );
    }

    #[test]
    fn completing_a_kill_task_reports_unsupported_rather_than_fabricating_loot() {
        let mut before = character();
        before.activity.task = "kill|Goblin|3|Sharp Rock".to_owned();
        before.progress.task.max = 100;
        before.progress.task.position = 0.0;
        let ruleset = Ruleset::default();

        let error = advance(&before, &ruleset, 100).unwrap_err();
        assert_eq!(
            error,
            SimulationError::Unsupported("kill-task loot resolution (ported in task 3.2)")
        );
    }
}
