use std::{fs, path::Path};

use gyrognome::fixtures::validate_fixture;

#[test]
fn committed_reference_fixtures_are_safe() {
    for entry in fs::read_dir("tests/fixtures").unwrap() {
        let path = entry.unwrap().path();
        validate_fixture(&path, &fs::read_to_string(&path).unwrap()).unwrap();
    }
}

#[test]
fn paired_timing_checkpoints_are_credential_free() {
    for fixture in [
        "checkpoint-incomplete-advancement.json",
        "checkpoint-completed-task.json",
    ] {
        let path = Path::new("tests/fixtures").join(fixture);
        let content = fs::read_to_string(&path).unwrap();
        validate_fixture(&path, &content).unwrap();
        assert!(!content.to_ascii_lowercase().contains("passkey"));
    }
}

#[test]
fn synthetic_report_trace_is_credential_free_and_unsigned() {
    let path = Path::new("tests/fixtures/report-trace-synthetic.json");
    let content = fs::read_to_string(path).unwrap();
    validate_fixture(path, &content).unwrap();
    assert!(!content.to_ascii_lowercase().contains("passkey"));
    assert!(!(content.contains("cmd=") && content.contains("&p=")));
}

#[test]
fn enrollment_evidence_rejects_response_bodies_and_raw_browser_data() {
    for content in [
        r#"{"response": "unsafe"}"#,
        r#"{"response_body": "unsafe"}"#,
        r#"{"raw_save": "unsafe"}"#,
        r#"{"profile": "unsafe"}"#,
    ] {
        assert!(validate_fixture(Path::new("enrollment-evidence.json"), content).is_err());
    }
}
