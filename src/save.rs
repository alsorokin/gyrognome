use std::{fs, path::Path};

use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::Value;
use thiserror::Error;

use crate::state::Character;

#[derive(Debug, Error)]
pub enum SaveError {
    #[error("could not read save: {0}")]
    Read(#[from] std::io::Error),
    #[error("save is not valid Base64")]
    Base64(#[from] base64::DecodeError),
    #[error("save JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("save is missing or has an invalid required field: {0}")]
    RequiredField(String),
}

impl SaveError {
    pub(crate) fn invalid(field: impl Into<String>) -> Self {
        Self::RequiredField(field.into())
    }
}

pub fn import_file(path: &Path) -> Result<Character, SaveError> {
    import_text(&fs::read_to_string(path)?)
}

pub fn import_text(text: &str) -> Result<Character, SaveError> {
    let decoded = STANDARD.decode(text.trim())?;
    Character::from_document(serde_json::from_slice::<Value>(&decoded)?)
}

pub fn export(character: &Character) -> Result<String, SaveError> {
    Ok(STANDARD.encode(serde_json::to_vec(&character.document)?))
}
