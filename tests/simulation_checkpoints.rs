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
fn loads_the_completed_task_checkpoint_committed_for_task_3_1() {
    // Task-completion dispatch (queue advancement, XP/quest/plot gating) is
    // ported in task 3.1; until then this checkpoint is only schema-checked
    // here. Its exact-match replay assertion is added alongside that task, as
    // described in tasks.md 3.1: "verify a completed-task checkpoint updates
    // task count, elapsed time, activity, and random state exactly."
    let checkpoint =
        checkpoint::load(Path::new("tests/fixtures/checkpoint-completed-task.json")).unwrap();
    assert_eq!(checkpoint.advancement_ms, vec![9451]);
    assert_eq!(checkpoint.initial.activity.tasks, 1);
    assert_eq!(checkpoint.expected.activity.tasks, 2);
    assert_eq!(checkpoint.expected.activity.elapsed, 12);
    // The completed task was not a "kill" task and the next queued item does
    // not require a random draw either, so this transition happens to leave
    // the Alea continuation untouched; it still exercises queue/task-count/
    // elapsed/plot bookkeeping once 3.1 lands.
    assert_eq!(checkpoint.expected.seed.0, checkpoint.initial.seed.0);
}
