use std::{fs, path::Path};

use base64::{Engine, engine::general_purpose::STANDARD};
use gyrognome::{
    checkpoint,
    fixtures::validate_fixture,
    protocol::{ConstructedReport, REVISION, progress_report_for_event},
    ruleset,
    save::import_text,
    simulation::{
        ExplicitReportAction, ReportEvent, ReportTrigger, advance, advance_with_trace,
        explicit_report_events,
    },
};
use serde::Deserialize;
use serde_json::to_value;

const SYNTHETIC_VALIDATOR_SALT: i32 = 4242;

#[derive(Debug, Deserialize)]
struct TraceSource {
    revision: String,
    content_sha256: String,
    derivation: String,
    validator_salt: i32,
}

#[derive(Debug, Deserialize)]
struct TraceFixture {
    source: TraceSource,
    events: Vec<ReportEvent>,
    reports: Vec<ConstructedReport>,
}

fn reference_character() -> gyrognome::state::Character {
    import_text(&STANDARD.encode(include_str!("fixtures/reference-save.json"))).unwrap()
}

fn capture_synthetic_trace() -> Vec<ReportEvent> {
    let reference = reference_character();
    let mut initial_load = reference.clone();
    initial_load.activity.elapsed = 0;
    let mut level = checkpoint::load(Path::new("tests/fixtures/checkpoint-level-up.json")).unwrap();
    let mut act = checkpoint::load(Path::new("tests/fixtures/checkpoint-act.json")).unwrap();
    level.initial.online = reference.online.clone();
    act.initial.online = reference.online.clone();

    let mut events = explicit_report_events(
        &initial_load,
        [ExplicitReportAction::InitialLoad {
            motto: "Synthetic motto".to_owned(),
        }],
    );
    events.extend(
        advance_with_trace(&level.initial, &ruleset::BUNDLED, 1_000, "Synthetic motto")
            .unwrap()
            .events,
    );
    events.extend(
        advance_with_trace(&act.initial, &ruleset::BUNDLED, 1_000, "Synthetic motto")
            .unwrap()
            .events,
    );
    events.extend(explicit_report_events(
        &reference,
        [
            ExplicitReportAction::ManualBrag {
                motto: "Synthetic motto".to_owned(),
            },
            ExplicitReportAction::MottoChange {
                motto: "Changed synthetic motto".to_owned(),
            },
        ],
    ));
    events
}

#[test]
fn replays_the_synthetic_report_trace_and_protocol_output_exactly() {
    let path = Path::new("tests/fixtures/report-trace-synthetic.json");
    let content = fs::read_to_string(path).unwrap();
    validate_fixture(path, &content).unwrap();
    let fixture: TraceFixture = serde_json::from_str(&content).unwrap();

    assert_eq!(fixture.source.revision, REVISION);
    assert_eq!(
        fixture.source.content_sha256,
        ruleset::SOURCE_CONTENT_SHA256
    );
    assert_eq!(
        fixture.source.derivation,
        "committed synthetic reference state and bundled static ruleset"
    );
    assert_eq!(fixture.source.validator_salt, SYNTHETIC_VALIDATOR_SALT);

    let events = capture_synthetic_trace();
    assert_eq!(
        to_value(&events).unwrap(),
        to_value(&fixture.events).unwrap()
    );
    assert_eq!(
        events
            .iter()
            .map(|event| event.trigger.code())
            .collect::<Vec<_>>(),
        vec!['s', 'l', 'a', 'b', 'm']
    );
    assert_eq!(
        events
            .iter()
            .map(|event| progress_report_for_event(event, SYNTHETIC_VALIDATOR_SALT).unwrap())
            .collect::<Vec<_>>(),
        fixture.reports
    );
}

#[test]
fn trace_snapshots_preserve_browser_report_call_sites_and_final_state() {
    let reference = reference_character();
    let mut state = checkpoint::load(Path::new("tests/fixtures/checkpoint-level-up.json"))
        .unwrap()
        .initial;
    state.online = reference.online;
    state.queue = vec!["plot|1|Loading".to_owned()];
    state.stats.best = "STR".to_owned();
    state.beststat = "STR 1".to_owned();

    let traced = advance_with_trace(&state, &ruleset::BUNDLED, 1_000, "Trace motto").unwrap();
    assert_eq!(
        traced
            .events
            .iter()
            .map(|event| event.trigger)
            .collect::<Vec<_>>(),
        vec![ReportTrigger::LevelUp, ReportTrigger::ActCompletion]
    );
    assert_eq!(
        to_value(&traced.state).unwrap(),
        to_value(advance(&state, &ruleset::BUNDLED, 1_000).unwrap()).unwrap()
    );

    let level = &traced.events[0].snapshot.character;
    let act = &traced.events[1].snapshot.character;
    for snapshot in [level, act, &traced.state] {
        assert_eq!(snapshot.stats.best, "CHA");
        assert_eq!(snapshot.beststat, "CHA 16");
    }
    assert_eq!(level.traits.level, 2);
    assert_eq!(level.inventory.len(), 1, "level report precedes kill loot");
    assert_eq!(
        act.inventory.len(),
        2,
        "act report follows completion effects"
    );
    assert_eq!(act.plot.act, 1);
    assert_eq!(act.activity.kill, "Executing a Goblin...");
}

#[test]
fn explicit_actions_are_credential_free_and_capture_their_input_motto() {
    let mut character = reference_character();
    character.activity.elapsed = 0;
    character.stats.best = "STR".to_owned();
    character.beststat = "STR 1".to_owned();
    character.stats.wisdom = 80.0;
    let events = explicit_report_events(
        &character,
        [
            ExplicitReportAction::InitialLoad {
                motto: "First motto".to_owned(),
            },
            ExplicitReportAction::ManualBrag {
                motto: "First motto".to_owned(),
            },
            ExplicitReportAction::MottoChange {
                motto: "Second motto".to_owned(),
            },
        ],
    );
    assert_eq!(
        events.iter().map(|event| event.trigger).collect::<Vec<_>>(),
        vec![
            ReportTrigger::InitialLoad,
            ReportTrigger::ManualBrag,
            ReportTrigger::MottoChange
        ]
    );
    assert_eq!(events[2].motto, "Second motto");
    assert_eq!(
        to_value(&events[0].snapshot.character).unwrap(),
        to_value(&events[1].snapshot.character).unwrap()
    );
    assert!(events[0].snapshot.character.document.is_null());
    for event in &events {
        assert_eq!(event.snapshot.character.stats.best, "WIS");
        assert_eq!(event.snapshot.character.beststat, "WIS 80");
    }
    assert_eq!(character.beststat, "STR 1");

    let serialized = serde_json::to_string(&events).unwrap();
    for prohibited in ["passkey", "4242", "cmd=", "&p="] {
        assert!(
            !serialized.to_ascii_lowercase().contains(prohibited),
            "serialized trace must not contain {prohibited}"
        );
    }
}

#[test]
fn trace_events_match_browser_online_and_initial_load_conditions() {
    let mut character = reference_character();
    character.activity.elapsed = 1;
    assert!(
        explicit_report_events(
            &character,
            [ExplicitReportAction::InitialLoad {
                motto: "Too late".to_owned(),
            }],
        )
        .is_empty()
    );

    character.online = None;
    assert!(
        explicit_report_events(
            &character,
            [
                ExplicitReportAction::ManualBrag {
                    motto: "Offline".to_owned(),
                },
                ExplicitReportAction::MottoChange {
                    motto: "Still offline".to_owned(),
                },
            ],
        )
        .is_empty()
    );
}
