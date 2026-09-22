use base64::{Engine, engine::general_purpose::STANDARD};
use gyrognome::{
    ruleset,
    save::{export, import_text},
    simulation,
};
use serde_json::{Value, json};

fn import_fixture() -> gyrognome::state::Character {
    import_text(&STANDARD.encode(include_str!("fixtures/reference-save.json"))).unwrap()
}

#[test]
fn imports_complete_ordered_canonical_state() {
    let character = import_fixture();
    assert_eq!(character.traits.name, "Reference Hero");
    assert_eq!(character.stats.intelligence, 13.0);
    assert_eq!(character.activity.elapsed, 42);
    assert_eq!(character.progress.task.position, 5.0);
    assert_eq!(character.inventory[1].name, "goblin ear");
    assert_eq!(character.spells[1].rank, "II");
    assert_eq!(character.quests[1], "Fetch me an anvil");
    assert_eq!(character.online.unwrap().realm, "Alpaquil");
    assert_eq!(character.profile.motto, "");
    assert_eq!(character.profile.guild, "");
}

#[test]
fn imports_optional_online_profile_values() {
    let mut document: Value =
        serde_json::from_str(include_str!("fixtures/reference-save.json")).unwrap();
    document["motto"] = json!("Progress through persistence");
    document["guild"] = json!("Gnomes");

    let character = import_text(&STANDARD.encode(serde_json::to_vec(&document).unwrap())).unwrap();

    assert_eq!(character.profile.motto, "Progress through persistence");
    assert_eq!(character.profile.guild, "Gnomes");
    assert!(
        character
            .summary()
            .contains("Motto: Progress through persistence")
    );
    assert!(character.summary().contains("Guild: Gnomes"));
    let inspected = serde_json::to_value(character).unwrap();
    assert_eq!(
        inspected["profile"]["motto"],
        "Progress through persistence"
    );
    assert_eq!(inspected["profile"]["guild"], "Gnomes");
}

#[test]
fn rejects_invalid_present_online_profile_values() {
    for field in ["motto", "guild"] {
        let mut document: Value =
            serde_json::from_str(include_str!("fixtures/reference-save.json")).unwrap();
        document[field] = json!(42);

        let error =
            import_text(&STANDARD.encode(serde_json::to_vec(&document).unwrap())).unwrap_err();

        assert!(error.to_string().contains(field));
    }
}

#[test]
fn imports_a_fresh_browser_character_missing_quest_fields() {
    // A brand-new browser character never assigns `questmonsterindex` or
    // `bestquest` until a monster-kill quest exists or a quest completes, so
    // the browser's `JSON.stringify` omits them entirely (observed via a
    // disposable Playwright session; see
    // tests/fixtures/checkpoint-incomplete-advancement.json). Importing must
    // default them rather than reject the save.
    let mut document: Value =
        serde_json::from_str(include_str!("fixtures/reference-save.json")).unwrap();
    document
        .as_object_mut()
        .unwrap()
        .remove("questmonsterindex");
    document.as_object_mut().unwrap().remove("bestquest");
    let character = import_text(&STANDARD.encode(serde_json::to_vec(&document).unwrap())).unwrap();
    assert_eq!(character.activity.questmonsterindex, 0);
    assert_eq!(character.bestquest, "");
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
        "Online Profile",
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

#[test]
fn simulation_leaves_imported_document_and_inspection_output_unchanged() {
    let character = import_fixture();
    let document = character.document.clone();
    let inspection = character.summary();

    let _simulated = simulation::advance(&character, &ruleset::BUNDLED, 40).unwrap();

    assert_eq!(character.document, document);
    assert_eq!(character.summary(), inspection);
}
