//! Deterministic simulation conformance checkpoints.
//!
//! A checkpoint states a synthetic canonical initial state, the ruleset
//! revision it was captured against, an explicit sequence of
//! elapsed-millisecond advancement steps, and the exact resulting canonical
//! state the browser client produced for that input. Since [`crate::state`]
//! unifies the parsed character's Alea state with the RNG continuation type
//! (see task 1.1), the expected Alea continuation is simply the `seed` field
//! of the expected canonical state; no separate representation is needed.
//!
//! Replaying a checkpoint (applying [`crate::simulation::advance`] once per
//! `advancement_ms` step to `initial` and comparing the result against
//! `expected`) is how simulation conformance with the browser client is
//! verified.

use std::{fs, path::Path};

use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

use crate::{
    fixtures::{FixtureSafetyError, validate_fixture},
    save::SaveError,
    state::Character,
};

#[derive(Debug, Deserialize)]
struct RawCheckpoint {
    ruleset: RawRulesetReference,
    advancement_ms: Vec<u64>,
    initial: Value,
    expected: Value,
}

#[derive(Debug, Deserialize)]
struct RawRulesetReference {
    revision: String,
    content_sha256: String,
}

/// A parsed, ready-to-replay conformance checkpoint.
#[derive(Debug, Clone)]
pub struct Checkpoint {
    /// The bundled ruleset revision the checkpoint was captured against (see
    /// [`crate::ruleset::SOURCE_REVISION`]).
    pub ruleset_revision: String,
    /// The bundled ruleset content hash the checkpoint was captured against
    /// (see [`crate::ruleset::SOURCE_CONTENT_SHA256`]).
    pub ruleset_content_sha256: String,
    /// The sequence of caller-supplied elapsed-millisecond advancement steps
    /// to apply, in order, to `initial`.
    pub advancement_ms: Vec<u64>,
    /// The canonical state to begin replay from.
    pub initial: Character,
    /// The canonical state (including Alea continuation) the browser client
    /// produced after applying `advancement_ms` to `initial`.
    pub expected: Character,
}

#[derive(Debug, Error)]
pub enum CheckpointError {
    #[error("could not read checkpoint fixture: {0}")]
    Read(#[from] std::io::Error),
    #[error("checkpoint fixture failed a safety check: {0}")]
    Unsafe(#[from] FixtureSafetyError),
    #[error("checkpoint fixture JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("checkpoint fixture state is invalid: {0}")]
    State(#[from] SaveError),
}

/// Loads and safety-checks a checkpoint fixture from `path`.
///
/// This always validates the fixture with [`validate_fixture`] before
/// parsing, rejecting a `.pqw` player-save extension, a signed leaderboard
/// request, or a browser profile path, regardless of caller.
pub fn load(path: &Path) -> Result<Checkpoint, CheckpointError> {
    let text = fs::read_to_string(path)?;
    validate_fixture(path, &text)?;
    let raw: RawCheckpoint = serde_json::from_str(&text)?;
    Ok(Checkpoint {
        ruleset_revision: raw.ruleset.revision,
        ruleset_content_sha256: raw.ruleset.content_sha256,
        advancement_ms: raw.advancement_ms,
        initial: Character::from_document(raw.initial)?,
        expected: Character::from_document(raw.expected)?,
    })
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    fn temp_path(name: &str) -> std::path::PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let unique = format!(
            "{}-{}-{name}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        std::env::temp_dir().join(unique)
    }

    fn write(path: &Path, content: &str) {
        fs::write(path, content).unwrap();
    }

    #[test]
    fn loads_a_well_formed_synthetic_checkpoint() {
        let initial: Value =
            serde_json::from_str(include_str!("../tests/fixtures/reference-save.json")).unwrap();
        let checkpoint = serde_json::json!({
            "ruleset": {"revision": "6", "content_sha256": "test"},
            "advancement_ms": [0],
            "initial": initial,
            "expected": initial,
        });
        let path = temp_path("checkpoint.json");
        write(&path, &checkpoint.to_string());

        let loaded = load(&path).unwrap();
        fs::remove_file(&path).unwrap();

        assert_eq!(loaded.ruleset_revision, "6");
        assert_eq!(loaded.advancement_ms, vec![0]);
        assert_eq!(loaded.initial.traits.name, loaded.expected.traits.name);
    }

    #[test]
    fn rejects_a_checkpoint_saved_with_the_player_save_extension() {
        let path = temp_path("checkpoint.pqw");
        write(&path, "{}");
        let error = load(&path).unwrap_err();
        fs::remove_file(&path).unwrap();
        assert!(matches!(
            error,
            CheckpointError::Unsafe(FixtureSafetyError::PlayerSave)
        ));
    }

    #[test]
    fn rejects_a_checkpoint_containing_a_signed_leaderboard_request() {
        let path = temp_path("checkpoint.json");
        write(&path, "cmd=b&t=l&p=123");
        let error = load(&path).unwrap_err();
        fs::remove_file(&path).unwrap();
        assert!(matches!(
            error,
            CheckpointError::Unsafe(FixtureSafetyError::SignedRequest)
        ));
    }

    #[test]
    fn rejects_a_checkpoint_containing_a_browser_profile_path() {
        let path = temp_path("checkpoint.json");
        write(&path, ".playwright-mcp/Default/profile");
        let error = load(&path).unwrap_err();
        fs::remove_file(&path).unwrap();
        assert!(matches!(
            error,
            CheckpointError::Unsafe(FixtureSafetyError::BrowserProfile)
        ));
    }
}
