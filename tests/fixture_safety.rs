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
