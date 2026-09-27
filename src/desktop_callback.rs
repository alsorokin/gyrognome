use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    compatibility::{DesktopCanonicalState, DesktopRandomState},
    desktop_rules::DelphiRandom,
    desktop_simulation::{DesktopProgressionError, DesktopProgressionHooks, complete_task},
};

pub const MAX_CALLBACK_ELAPSED_MS: i64 = 100;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopCallbackCheckpoint {
    pub state: DesktopCanonicalState,
    pub random: DesktopRandomState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopCallbackObservation {
    pub credited_milliseconds: u64,
    pub completion_dispatched: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DesktopCallbackError {
    #[error("desktop task bar maximum must be positive")]
    InvalidTaskMaximum,
    #[error("desktop task bar position exceeds its maximum")]
    InvalidTaskPosition,
    #[error("desktop task completion failed: {0}")]
    Completion(&'static str),
    #[error(transparent)]
    Progression(#[from] DesktopProgressionError),
}

impl DesktopCallbackCheckpoint {
    pub fn apply_callback(
        &mut self,
        elapsed_milliseconds: i64,
        complete: impl FnOnce(
            &mut DesktopCanonicalState,
            &mut DelphiRandom,
        ) -> Result<(), DesktopCallbackError>,
    ) -> Result<DesktopCallbackObservation, DesktopCallbackError> {
        validate_task_bar(self)?;
        if self.state.bars.task.position == self.state.bars.task.maximum {
            let mut next = self.clone();
            let mut random = DelphiRandom::from_state(next.random);
            complete(&mut next.state, &mut random)?;
            next.random = random.state();
            validate_task_bar(&next)?;
            *self = next;
            return Ok(DesktopCallbackObservation {
                credited_milliseconds: 0,
                completion_dispatched: true,
            });
        }

        let credited_milliseconds = elapsed_milliseconds.clamp(0, MAX_CALLBACK_ELAPSED_MS) as u64;
        self.state.bars.task.position = self
            .state
            .bars
            .task
            .position
            .saturating_add(credited_milliseconds)
            .min(self.state.bars.task.maximum);
        Ok(DesktopCallbackObservation {
            credited_milliseconds,
            completion_dispatched: false,
        })
    }

    pub fn apply_callbacks(
        &mut self,
        elapsed_milliseconds: &[i64],
        mut complete: impl FnMut(
            &mut DesktopCanonicalState,
            &mut DelphiRandom,
        ) -> Result<(), DesktopCallbackError>,
    ) -> Result<Vec<DesktopCallbackObservation>, DesktopCallbackError> {
        elapsed_milliseconds
            .iter()
            .copied()
            .map(|elapsed| self.apply_callback(elapsed, &mut complete))
            .collect()
    }

    pub fn apply_progression_callback(
        &mut self,
        elapsed_milliseconds: i64,
        hooks: &mut impl DesktopProgressionHooks,
    ) -> Result<DesktopCallbackObservation, DesktopCallbackError> {
        self.apply_callback(elapsed_milliseconds, |state, random| {
            complete_task(state, random, hooks)?;
            Ok(())
        })
    }
}

fn validate_task_bar(checkpoint: &DesktopCallbackCheckpoint) -> Result<(), DesktopCallbackError> {
    let task = &checkpoint.state.bars.task;
    if task.maximum == 0 {
        return Err(DesktopCallbackError::InvalidTaskMaximum);
    }
    if task.position > task.maximum {
        return Err(DesktopCallbackError::InvalidTaskPosition);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        compatibility::DesktopCanonicalState,
        desktop_save::{
            DesktopQuestMarker, DesktopValidatedBar, DesktopValidatedBars, DesktopValidatedProfile,
        },
    };

    use super::*;

    fn checkpoint(position: u64, maximum: u64) -> DesktopCallbackCheckpoint {
        DesktopCallbackCheckpoint {
            state: DesktopCanonicalState {
                traits: Vec::new(),
                stats: Vec::new(),
                equipment: Vec::new(),
                inventory: Vec::new(),
                spells: Vec::new(),
                plots: Vec::new(),
                quests: Vec::new(),
                current_task: "task".to_owned(),
                quest: DesktopQuestMarker::None,
                queue: Vec::new(),
                activity: "Working".to_owned(),
                bars: DesktopValidatedBars {
                    experience: DesktopValidatedBar {
                        position: 0,
                        maximum: 1,
                    },
                    encumbrance: DesktopValidatedBar {
                        position: 0,
                        maximum: 1,
                    },
                    plot: DesktopValidatedBar {
                        position: 0,
                        maximum: 1,
                    },
                    quest: DesktopValidatedBar {
                        position: 0,
                        maximum: 1,
                    },
                    task: DesktopValidatedBar { position, maximum },
                },
                prized_equipment: 0,
                game_style: 3,
                profile: DesktopValidatedProfile {
                    motto: String::new(),
                    guild: String::new(),
                },
            },
            random: DesktopRandomState(0x1234_5678),
        }
    }

    fn select_next_task(
        state: &mut DesktopCanonicalState,
        random: &mut DelphiRandom,
    ) -> Result<(), DesktopCallbackError> {
        state.current_task = "next".to_owned();
        state.activity = format!("Next {}", random.bounded(10).unwrap());
        state.bars.task = DesktopValidatedBar {
            position: 0,
            maximum: 9_000,
        };
        Ok(())
    }

    #[test]
    fn requires_sixty_advancement_callbacks_and_a_completion_callback() {
        let mut checkpoint = checkpoint(0, 6_000);
        let observations = checkpoint
            .apply_callbacks(&[100; 60], select_next_task)
            .unwrap();
        assert!(
            observations
                .iter()
                .all(|observation| !observation.completion_dispatched)
        );
        assert_eq!(checkpoint.state.bars.task.position, 6_000);
        assert_eq!(checkpoint.state.current_task, "task");

        let completion = checkpoint.apply_callback(100, select_next_task).unwrap();
        assert_eq!(
            completion,
            DesktopCallbackObservation {
                credited_milliseconds: 0,
                completion_dispatched: true,
            }
        );
        assert_eq!(checkpoint.state.bars.task.position, 0);
        assert_eq!(checkpoint.state.current_task, "next");
    }

    #[test]
    fn clamps_negative_zero_and_delayed_elapsed_without_carrying_excess() {
        let mut checkpoint = checkpoint(0, 6_000);
        let observations = checkpoint
            .apply_callbacks(&[-50, 0, 10_000], select_next_task)
            .unwrap();
        assert_eq!(
            observations,
            [
                DesktopCallbackObservation {
                    credited_milliseconds: 0,
                    completion_dispatched: false,
                },
                DesktopCallbackObservation {
                    credited_milliseconds: 0,
                    completion_dispatched: false,
                },
                DesktopCallbackObservation {
                    credited_milliseconds: 100,
                    completion_dispatched: false,
                },
            ]
        );
        assert_eq!(checkpoint.state.bars.task.position, 100);
    }

    #[test]
    fn callback_batching_matches_one_at_a_time_execution() {
        let elapsed = [250, 75, -1, 100, 10_000, 40];
        let mut batched = checkpoint(5_800, 6_000);
        let batched_observations = batched.apply_callbacks(&elapsed, select_next_task).unwrap();

        let mut sequential = checkpoint(5_800, 6_000);
        let sequential_observations = elapsed
            .into_iter()
            .map(|value| sequential.apply_callback(value, select_next_task).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(batched_observations, sequential_observations);
        assert_eq!(batched, sequential);
    }

    #[test]
    fn restored_full_bar_dispatches_once_with_identical_random_continuation() {
        let pending = checkpoint(6_000, 6_000);
        let encoded = serde_json::to_string(&pending).unwrap();
        let mut restored: DesktopCallbackCheckpoint = serde_json::from_str(&encoded).unwrap();
        let mut uninterrupted = pending;

        let restored_completion = restored.apply_callback(100, select_next_task).unwrap();
        let uninterrupted_completion = uninterrupted.apply_callback(100, select_next_task).unwrap();
        assert_eq!(restored_completion, uninterrupted_completion);
        assert_eq!(restored, uninterrupted);

        let next = restored.apply_callback(100, select_next_task).unwrap();
        assert!(!next.completion_dispatched);
        assert_eq!(restored.state.bars.task.position, 100);
        assert_eq!(restored.random, uninterrupted.random);
    }

    #[test]
    fn rejects_invalid_task_bars_and_rolls_back_failed_completion() {
        assert_eq!(
            checkpoint(0, 0).apply_callback(100, select_next_task),
            Err(DesktopCallbackError::InvalidTaskMaximum)
        );
        assert_eq!(
            checkpoint(2, 1).apply_callback(100, select_next_task),
            Err(DesktopCallbackError::InvalidTaskPosition)
        );

        let mut pending = checkpoint(6_000, 6_000);
        let original = pending.clone();
        assert_eq!(
            pending.apply_callback(100, |state, random| {
                state.current_task = "partial".to_owned();
                random.bounded(10).unwrap();
                Err(DesktopCallbackError::Completion("synthetic failure"))
            }),
            Err(DesktopCallbackError::Completion("synthetic failure"))
        );
        assert_eq!(pending, original);
    }
}
