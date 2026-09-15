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
    RequiredField(&'static str),
}

pub fn import_file(path: &Path) -> Result<Character, SaveError> {
    import_text(&fs::read_to_string(path)?)
}

pub fn import_text(text: &str) -> Result<Character, SaveError> {
    let decoded = STANDARD.decode(text.trim())?;
    let document: Value = serde_json::from_slice(&decoded)?;
    character_from_document(document)
}

pub fn export(character: &Character) -> Result<String, SaveError> {
    Ok(STANDARD.encode(serde_json::to_vec(&character.document)?))
}

fn character_from_document(document: Value) -> Result<Character, SaveError> {
    let traits = document
        .get("Traits")
        .and_then(Value::as_object)
        .ok_or(SaveError::RequiredField("Traits"))?;
    let name = required_string(traits.get("Name"), "Traits.Name")?;
    let race = required_string(traits.get("Race"), "Traits.Race")?;
    let class = required_string(traits.get("Class"), "Traits.Class")?;
    let level = traits
        .get("Level")
        .and_then(Value::as_u64)
        .filter(|level| *level > 0)
        .ok_or(SaveError::RequiredField("Traits.Level"))?;

    required_array(&document, "dna")?;
    required_array(&document, "seed")?;
    required_object(&document, "Stats")?;
    required_array(&document, "Inventory")?;
    required_array(&document, "Spells")?;
    required_array(&document, "Quests")?;
    required_object(&document, "ExpBar")?;
    required_object(&document, "TaskBar")?;

    let online_realm = match document.get("online") {
        None => None,
        Some(online) => Some(required_string(online.get("realm"), "online.realm")?),
    };

    Ok(Character {
        document,
        name,
        race,
        class,
        level,
        online_realm,
    })
}

fn required_string(value: Option<&Value>, field: &'static str) -> Result<String, SaveError> {
    value
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(SaveError::RequiredField(field))
}

fn required_array(document: &Value, field: &'static str) -> Result<(), SaveError> {
    document
        .get(field)
        .and_then(Value::as_array)
        .map(|_| ())
        .ok_or(SaveError::RequiredField(field))
}

fn required_object(document: &Value, field: &'static str) -> Result<(), SaveError> {
    document
        .get(field)
        .and_then(Value::as_object)
        .map(|_| ())
        .ok_or(SaveError::RequiredField(field))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn document() -> Value {
        json!({
            "Traits": {"Name": "Reference", "Race": "Gyrognome", "Class": "Robot Monk", "Level": 1},
            "dna": [0.1, 0.2, 0.3, 1],
            "seed": [0.4, 0.5, 0.6, 2],
            "Stats": {}, "Inventory": [], "Spells": [], "Quests": [],
            "ExpBar": {}, "TaskBar": {}, "future-field": {"preserved": true}
        })
    }

    #[test]
    fn round_trips_unmodified_document() {
        let text = STANDARD.encode(serde_json::to_vec(&document()).unwrap());
        let imported = import_text(&text).unwrap();
        let exported = export(&imported).unwrap();
        let reimported = import_text(&exported).unwrap();
        assert_eq!(reimported.document["future-field"]["preserved"], true);
        assert_eq!(reimported.name, "Reference");
    }

    #[test]
    fn rejects_invalid_inputs() {
        assert!(import_text("not base64").is_err());
        assert!(import_text(&STANDARD.encode(b"{}")).is_err());
    }
}
