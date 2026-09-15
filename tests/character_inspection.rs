use base64::{Engine, engine::general_purpose::STANDARD};
use gyrognome::save::{export, import_text};
use serde_json::{Value, json};

fn import_fixture() -> gyrognome::state::Character {
    import_text(&STANDARD.encode(include_str!("fixtures/reference-save.json"))).unwrap()
}

#[test]
fn imports_complete_ordered_canonical_state() {
    let character = import_fixture();
    assert_eq!(character.traits.name, "Reference Hero");
    assert_eq!(character.stats.intelligence, 13);
    assert_eq!(character.activity.elapsed, 42);
    assert_eq!(character.progress.task.position, 5.0);
    assert_eq!(character.inventory[1].name, "goblin ear");
    assert_eq!(character.spells[1].rank, "II");
    assert_eq!(character.quests[1], "Fetch me an anvil");
    assert_eq!(character.online.unwrap().realm, "Alpaquil");
}

#[test]
fn imports_fractional_browser_progress_positions() {
    let mut document: Value =
        serde_json::from_str(include_str!("fixtures/reference-save.json")).unwrap();
    document["ExpBar"]["position"] = json!(379485.2000000185);
    let character = import_text(&STANDARD.encode(serde_json::to_vec(&document).unwrap())).unwrap();
    assert_eq!(character.progress.experience.position, 379485.2000000185);
}

#[test]
fn rejects_invalid_nested_fields_with_paths() {
    let mut document: Value =
        serde_json::from_str(include_str!("fixtures/reference-save.json")).unwrap();
    document["Inventory"][1][1] = json!("two");
    let error = import_text(&STANDARD.encode(serde_json::to_vec(&document).unwrap())).unwrap_err();
    assert!(error.to_string().contains("Inventory[1][1]"));
}

#[test]
fn json_and_text_inspection_redact_private_data() {
    let character = import_fixture();
    let json = serde_json::to_string(&character).unwrap();
    let text = character.summary();
    for output in [&json, &text] {
        assert!(!output.contains("4242"));
        assert!(!output.contains("unrecognized-future-field"));
        assert!(output.contains("Reference Hero"));
    }
    assert!(json.contains("\"Inventory\""));
    for section in [
        "Identity",
        "Attributes",
        "Activity",
        "Progress",
        "Equipment",
        "Inventory",
        "Spells",
        "Plot and Quests",
    ] {
        assert!(text.contains(section), "missing {section} section");
    }
}

#[test]
fn export_round_trip_preserves_unknown_document_data() {
    let character = import_fixture();
    let reimported = import_text(&export(&character).unwrap()).unwrap();
    assert_eq!(
        reimported.document["unrecognized-future-field"]["retained"],
        true
    );
}
