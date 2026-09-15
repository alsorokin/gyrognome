use std::path::Path;

use gyrognome::{checkpoint, ruleset, simulation};
use serde_json::to_value;

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

    let mut state = checkpoint.initial;
    for elapsed_ms in checkpoint.advancement_ms {
        state = simulation::advance(&state, &ruleset::BUNDLED, elapsed_ms).unwrap();
    }

    // Compare full serializable canonical state (not just a summary), per the
    // "Replaying a checkpoint" scenario: the resulting canonical state must
    // match exactly, including the Alea continuation carried in `seed`.
    assert_eq!(
        to_value(&state).unwrap(),
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

    let mut state = checkpoint.initial;
    for elapsed_ms in checkpoint.advancement_ms {
        state = simulation::advance(&state, &ruleset::BUNDLED, elapsed_ms).unwrap();
    }

    assert_eq!(
        to_value(&state).unwrap(),
        to_value(&checkpoint.expected).unwrap()
    );
}
