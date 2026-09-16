//! Test-only JSON adapter for the disposable Playwright conformance harness.
//!
//! This module is compiled only with the `conformance-bridge` feature. It
//! accepts a credential-free canonical state and emits a credential-free
//! successor state plus report events; it never reads a save, opens storage,
//! or constructs a request URL.

use std::io::{self, Read};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::{
    ruleset,
    save::SaveError,
    simulation::{ExplicitReportAction, ReportEvent, advance_with_trace, explicit_report_events},
    state::Character,
};

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("could not read bridge input: {0}")]
    Read(#[from] io::Error),
    #[error("bridge input or output JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("bridge state is invalid: {0}")]
    State(#[from] SaveError),
    #[error("bridge input must not contain {0}")]
    Sensitive(&'static str),
    #[error("bridge input must specify exactly one credential-free state source")]
    StateSource,
    #[error(transparent)]
    Simulation(#[from] crate::simulation::SimulationError),
}

#[derive(Debug, Deserialize)]
enum SyntheticFixture {
    #[serde(rename = "report-trace-synthetic")]
    ReportTraceSynthetic,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BridgeInput {
    #[serde(default)]
    character: Option<Value>,
    #[serde(default)]
    fixture: Option<SyntheticFixture>,
    #[serde(default)]
    advancement_ms: Vec<u64>,
    #[serde(default)]
    motto: String,
    #[serde(default)]
    actions: Vec<ExplicitReportAction>,
}

#[derive(Debug, Serialize)]
struct BridgeOutput {
    state: Character,
    events: Vec<ReportEvent>,
}

/// Processes a bridge payload without accessing a save document or transport.
pub fn process(input: &str) -> Result<String, BridgeError> {
    reject_sensitive(input)?;
    let input: BridgeInput = serde_json::from_str(input)?;
    let character = match (input.character, input.fixture) {
        (Some(character), None) => character,
        (None, Some(SyntheticFixture::ReportTraceSynthetic)) => serde_json::from_str::<Value>(
            include_str!("../tests/fixtures/report-trace-synthetic.json"),
        )?["events"][0]["snapshot"]["character"]
            .clone(),
        _ => return Err(BridgeError::StateSource),
    };
    reject_sensitive_value(&character)?;

    let mut state = if character.get("activity").is_some() {
        serde_json::from_value(character)?
    } else {
        Character::from_document(character)?
    };
    state.document = Value::Null;
    let mut events = Vec::new();
    for elapsed_ms in input.advancement_ms {
        let result = advance_with_trace(&state, &ruleset::BUNDLED, elapsed_ms, &input.motto)?;
        state = result.state;
        events.extend(result.events);
    }
    events.extend(explicit_report_events(&state, input.actions));

    let output = serde_json::to_string(&BridgeOutput { state, events })?;
    reject_sensitive(&output)?;
    Ok(output)
}

/// Runs the adapter on standard input/output for the Playwright harness.
pub fn run_stdio() -> Result<(), BridgeError> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    println!("{}", process(&input)?);
    Ok(())
}

fn reject_sensitive(input: &str) -> Result<(), BridgeError> {
    let lower = input.to_ascii_lowercase();
    if lower.contains("passkey") {
        return Err(BridgeError::Sensitive("a passkey"));
    }
    if lower.contains("\"document\"") || lower.contains("original_document") {
        return Err(BridgeError::Sensitive("a save document"));
    }
    if lower.contains("cmd=") && lower.contains("&p=") {
        return Err(BridgeError::Sensitive("a complete signed request URL"));
    }
    if lower.contains(".pqw") {
        return Err(BridgeError::Sensitive("a player save reference"));
    }
    Ok(())
}

fn reject_sensitive_value(value: &Value) -> Result<(), BridgeError> {
    reject_sensitive(&serde_json::to_string(value)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use serde_json::{Value, json};

    use super::*;

    fn canonical_character() -> Value {
        let character = crate::save::import_text(
            &STANDARD.encode(include_str!("../tests/fixtures/reference-save.json")),
        )
        .unwrap();
        serde_json::to_value(character).unwrap()
    }

    #[test]
    fn accepts_the_committed_synthetic_trace_fixture_as_a_state_source() {
        let output = process(r#"{"fixture":"report-trace-synthetic"}"#).unwrap();
        let value: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value["state"]["Traits"]["Name"], "Reference Hero");
        assert!(value["events"].as_array().unwrap().is_empty());
    }

    #[test]
    fn advances_a_synthetic_canonical_state_and_returns_only_safe_fields() {
        let output = process(
            &json!({
                "character": canonical_character(),
                "advancement_ms": [100],
                "motto": "Synthetic motto",
                "actions": [{"ManualBrag": {"motto": "Synthetic motto"}}],
            })
            .to_string(),
        )
        .unwrap();
        let value: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value["events"][0]["trigger"], "b");
        assert!(!output.to_ascii_lowercase().contains("passkey"));
        assert!(!output.contains("4242"));
        assert!(!output.contains("cmd="));
        assert!(value["state"].get("document").is_none());
    }

    #[test]
    fn rejects_save_documents_credentials_and_signed_requests() {
        for unsafe_input in [
            r#"{"character":{"document":{}}}"#,
            r#"{"character":{"passkey":4242}}"#,
            r#"{"character":"https://example.invalid/?cmd=b&p=4242"}"#,
            r#"{"character":"subject.pqw"}"#,
        ] {
            assert!(process(unsafe_input).is_err(), "{unsafe_input}");
        }
    }
}
