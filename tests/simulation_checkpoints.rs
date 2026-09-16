use std::path::Path;

use gyrognome::{checkpoint, ruleset, simulation};
use serde_json::{Value, to_value};

fn replay(checkpoint: &checkpoint::Checkpoint, advancement_ms: &[u64]) -> Value {
    advancement_ms
        .iter()
        .try_fold(checkpoint.initial.clone(), |state, &elapsed_ms| {
            simulation::advance(&state, &ruleset::BUNDLED, elapsed_ms)
        })
        .map(|state| to_value(&state).unwrap())
        .unwrap()
}

#[test]
fn replays_the_incomplete_advancement_checkpoint_exactly() {
    let checkpoint = checkpoint::load(Path::new(
        "tests/fixtures/checkpoint-incomplete-advancement.json",
    ))
    .unwrap();
    assert_eq!(checkpoint.ruleset_revision, ruleset::SOURCE_REVISION);
    assert_eq!(
        checkpoint.ruleset_content_sha256,
        ruleset::SOURCE_CONTENT_SHA256
    );

    // Compare full serializable canonical state (not just a summary), per the
    // "Replaying a checkpoint" scenario: the resulting canonical state must
    // match exactly, including the Alea continuation carried in `seed`.
    assert_eq!(
        replay(&checkpoint, &checkpoint.advancement_ms),
        to_value(&checkpoint.expected).unwrap()
    );
}

#[test]
fn replays_the_completed_task_checkpoint_exactly() {
    // This checkpoint's completed task was a plain narrative task (not a
    // "kill" task), so it only exercises task-count/elapsed bookkeeping, the
    // plot-bar "gain || !act" advancement, and plain task-queue dequeuing —
    // all ported in task 3.1. It does not exercise kill-task loot, monster
    // task selection, or reward paths (tasks 3.2/3.3); those get their own
    // checkpoints when ported.
    let checkpoint =
        checkpoint::load(Path::new("tests/fixtures/checkpoint-completed-task.json")).unwrap();
    assert_eq!(checkpoint.ruleset_revision, ruleset::SOURCE_REVISION);
    assert_eq!(
        checkpoint.ruleset_content_sha256,
        ruleset::SOURCE_CONTENT_SHA256
    );

    assert_eq!(
        replay(&checkpoint, &checkpoint.advancement_ms),
        to_value(&checkpoint.expected).unwrap()
    );
}

#[test]
fn replays_each_browser_derived_checkpoint_exactly() {
    for fixture in [
        "checkpoint-incomplete-advancement.json",
        "checkpoint-completed-task.json",
        "checkpoint-level-up.json",
        "checkpoint-equipment.json",
        "checkpoint-inventory.json",
        "checkpoint-quest.json",
        "checkpoint-plot.json",
        "checkpoint-act.json",
    ] {
        let checkpoint = checkpoint::load(Path::new("tests/fixtures").join(fixture).as_path())
            .unwrap_or_else(|error| panic!("{fixture}: {error}"));
        let expected = to_value(&checkpoint.expected).unwrap();
        assert_eq!(
            replay(&checkpoint, &checkpoint.advancement_ms),
            expected,
            "{fixture}"
        );
        if !checkpoint.equivalent_advancement_ms.is_empty() {
            assert!(
                checkpoint
                    .equivalent_advancement_ms
                    .iter()
                    .all(|&elapsed_ms| elapsed_ms > 0),
                "{fixture} must use only nonzero partition durations"
            );
            assert_eq!(
                checkpoint.equivalent_advancement_ms.iter().sum::<u64>(),
                checkpoint.advancement_ms.iter().sum::<u64>(),
                "{fixture} must preserve total elapsed duration"
            );
            assert_eq!(
                replay(&checkpoint, &checkpoint.equivalent_advancement_ms),
                expected,
                "{fixture}"
            );
        }
    }
}
