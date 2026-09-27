use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::rng::AleaState;
use crate::{
    desktop_save::{
        DesktopAdaptations, DesktopQuestMarker, DesktopQueueCommand, DesktopValidatedBars,
        DesktopValidatedProfile, DesktopValidatedRow, DesktopValidatedSave,
    },
    newguy,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompatibilityProfile {
    #[serde(rename = "browser")]
    Browser,
    #[serde(rename = "desktop-6.4.4")]
    Desktop644,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DesktopRandomState(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "profile", content = "state")]
pub enum RandomContinuation {
    #[serde(rename = "browser")]
    Browser(AleaState),
    #[serde(rename = "desktop-6.4.4")]
    Desktop644(DesktopRandomState),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CompatibilityState {
    pub profile: CompatibilityProfile,
    pub random: RandomContinuation,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CompatibilityError {
    #[error("compatibility profile and random continuation do not match")]
    RandomProfileMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceFormat {
    BrowserJson,
    DesktopDelphiComponentStream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesktopLayout {
    SupportedComponentStream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesktopAdaptation {
    LegacyPrologue62,
    LegacyQuestPlaceholder,
    LoadSpellingPatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HistoricalValue {
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopSourceProvenance {
    pub source_format: SourceFormat,
    pub recognized_layout: DesktopLayout,
    pub adaptations: Vec<DesktopAdaptation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator_declared_origin: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopUnavailableHistory {
    pub original_random_continuation: HistoricalValue,
    pub birthday: HistoricalValue,
    pub seed_history: HistoricalValue,
    pub lifetime_tasks: HistoricalValue,
    pub lifetime_elapsed: HistoricalValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SinceImportCounters {
    pub tasks_completed: u64,
    pub elapsed_milliseconds: u64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesktopAdvancementProvenance {
    #[default]
    Unadvanced,
    LocalOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopImportMetadata {
    pub provenance: DesktopSourceProvenance,
    pub unavailable_history: DesktopUnavailableHistory,
    pub measured_since_import: SinceImportCounters,
    #[serde(default)]
    pub advancement_provenance: DesktopAdvancementProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopCanonicalState {
    pub traits: Vec<DesktopValidatedRow>,
    pub stats: Vec<DesktopValidatedRow>,
    pub equipment: Vec<DesktopValidatedRow>,
    pub inventory: Vec<DesktopValidatedRow>,
    pub spells: Vec<DesktopValidatedRow>,
    pub plots: Vec<DesktopValidatedRow>,
    pub quests: Vec<DesktopValidatedRow>,
    pub current_task: String,
    pub quest: DesktopQuestMarker,
    pub queue: Vec<DesktopQueueCommand>,
    pub activity: String,
    pub bars: DesktopValidatedBars,
    pub prized_equipment: usize,
    pub game_style: u32,
    pub profile: DesktopValidatedProfile,
}

impl From<&DesktopValidatedSave> for DesktopCanonicalState {
    fn from(save: &DesktopValidatedSave) -> Self {
        Self {
            traits: save.traits.clone(),
            stats: save.stats.clone(),
            equipment: save.equipment.clone(),
            inventory: save.inventory.clone(),
            spells: save.spells.clone(),
            plots: save.plots.clone(),
            quests: save.quests.clone(),
            current_task: save.current_task.clone(),
            quest: save.quest.clone(),
            queue: save.queue.clone(),
            activity: save.activity.clone(),
            bars: save.bars.clone(),
            prized_equipment: save.prized_equipment,
            game_style: save.game_style,
            profile: save.profile.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "profile", content = "metadata")]
pub enum ImportMetadata {
    #[serde(rename = "browser")]
    Browser { source_format: SourceFormat },
    #[serde(rename = "desktop-6.4.4")]
    Desktop(DesktopImportMetadata),
}

impl DesktopImportMetadata {
    pub fn from_validated(adaptations: &DesktopAdaptations) -> Self {
        let mut recorded = Vec::new();
        if adaptations.legacy_prologue_62 {
            recorded.push(DesktopAdaptation::LegacyPrologue62);
        }
        if adaptations.legacy_quest_placeholder {
            recorded.push(DesktopAdaptation::LegacyQuestPlaceholder);
        }
        if adaptations.spelling_patch_applied {
            recorded.push(DesktopAdaptation::LoadSpellingPatch);
        }
        Self {
            provenance: DesktopSourceProvenance {
                source_format: SourceFormat::DesktopDelphiComponentStream,
                recognized_layout: DesktopLayout::SupportedComponentStream,
                adaptations: recorded,
                operator_declared_origin: None,
            },
            unavailable_history: DesktopUnavailableHistory {
                original_random_continuation: HistoricalValue::Unavailable,
                birthday: HistoricalValue::Unavailable,
                seed_history: HistoricalValue::Unavailable,
                lifetime_tasks: HistoricalValue::Unavailable,
                lifetime_elapsed: HistoricalValue::Unavailable,
            },
            measured_since_import: SinceImportCounters {
                tasks_completed: 0,
                elapsed_milliseconds: 0,
            },
            advancement_provenance: DesktopAdvancementProvenance::Unadvanced,
        }
    }
}

pub fn initialize_desktop_registration_random(
    source: &mut impl newguy::RandomSource,
) -> Result<DesktopRandomState, newguy::NewGuyError> {
    source.next_u32().map(DesktopRandomState)
}

impl CompatibilityState {
    pub fn validate(self) -> Result<Self, CompatibilityError> {
        let matches = matches!(
            (self.profile, self.random),
            (
                CompatibilityProfile::Browser,
                RandomContinuation::Browser(_)
            ) | (
                CompatibilityProfile::Desktop644,
                RandomContinuation::Desktop644(_)
            )
        );
        if !matches {
            return Err(CompatibilityError::RandomProfileMismatch);
        }
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    struct Numbers {
        next: u32,
        calls: usize,
    }

    impl newguy::RandomSource for Numbers {
        fn next_u32(&mut self) -> Result<u32, newguy::NewGuyError> {
            self.calls += 1;
            Ok(self.next)
        }
    }

    #[test]
    fn accepts_matching_browser_and_desktop_continuations() {
        let browser = CompatibilityState {
            profile: CompatibilityProfile::Browser,
            random: RandomContinuation::Browser(AleaState([0.1, 0.2, 0.3, 1.0])),
        };
        let desktop = CompatibilityState {
            profile: CompatibilityProfile::Desktop644,
            random: RandomContinuation::Desktop644(DesktopRandomState(4_242)),
        };

        assert_eq!(browser.validate().unwrap(), browser);
        assert_eq!(desktop.validate().unwrap(), desktop);
    }

    #[test]
    fn rejects_cross_profile_random_continuations() {
        for state in [
            CompatibilityState {
                profile: CompatibilityProfile::Browser,
                random: RandomContinuation::Desktop644(DesktopRandomState(4_242)),
            },
            CompatibilityState {
                profile: CompatibilityProfile::Desktop644,
                random: RandomContinuation::Browser(AleaState([0.1, 0.2, 0.3, 1.0])),
            },
        ] {
            assert_eq!(
                state.validate(),
                Err(CompatibilityError::RandomProfileMismatch)
            );
        }
    }

    #[test]
    fn rejects_unknown_profile_and_random_variants() {
        let unknown_profile = json!({
            "profile": "future-client",
            "random": {
                "profile": "browser",
                "state": [0.1, 0.2, 0.3, 1.0]
            }
        });
        let unknown_random = json!({
            "profile": "browser",
            "random": {
                "profile": "future-random",
                "state": 42
            }
        });

        assert!(serde_json::from_value::<CompatibilityState>(unknown_profile).is_err());
        assert!(serde_json::from_value::<CompatibilityState>(unknown_random).is_err());
    }

    #[test]
    fn serializes_closed_profile_identity_and_random_variant() {
        let state = CompatibilityState {
            profile: CompatibilityProfile::Desktop644,
            random: RandomContinuation::Desktop644(DesktopRandomState(u32::MAX)),
        };

        assert_eq!(
            serde_json::to_value(state).unwrap(),
            json!({
                "profile": "desktop-6.4.4",
                "random": {
                    "profile": "desktop-6.4.4",
                    "state": u32::MAX
                }
            })
        );
    }

    #[test]
    fn desktop_import_metadata_marks_history_unavailable_without_writer_claims() {
        let metadata = DesktopImportMetadata::from_validated(&DesktopAdaptations {
            legacy_prologue_62: true,
            legacy_quest_placeholder: true,
            spelling_patch_applied: false,
        });
        let value = serde_json::to_value(metadata).unwrap();

        assert_eq!(
            value["provenance"]["source_format"],
            "desktop-delphi-component-stream"
        );
        assert_eq!(
            value["provenance"]["adaptations"],
            json!(["legacy-prologue62", "legacy-quest-placeholder"])
        );
        assert!(value["provenance"].get("writer_version").is_none());
        assert_eq!(
            value["unavailable_history"]["original_random_continuation"],
            "unavailable"
        );
        assert_eq!(value["measured_since_import"]["tasks_completed"], 0);
        assert_eq!(value["measured_since_import"]["elapsed_milliseconds"], 0);
        assert_eq!(value["advancement_provenance"], "unadvanced");
    }

    #[test]
    fn desktop_random_state_is_created_only_by_registration_initialization() {
        let mut source = Numbers {
            next: 0xf00d_cafe,
            calls: 0,
        };
        let metadata = DesktopImportMetadata::from_validated(&DesktopAdaptations {
            legacy_prologue_62: false,
            legacy_quest_placeholder: false,
            spelling_patch_applied: false,
        });

        assert_eq!(source.calls, 0);
        assert_eq!(
            metadata.unavailable_history.original_random_continuation,
            HistoricalValue::Unavailable
        );
        assert_eq!(
            initialize_desktop_registration_random(&mut source).unwrap(),
            DesktopRandomState(0xf00d_cafe)
        );
        assert_eq!(source.calls, 1);
    }
}
