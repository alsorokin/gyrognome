use std::fs;

use gyrognome::{
    protocol::{REVISION, lfsr},
    rng::{Alea, AleaState},
};
use serde_json::Value;

#[test]
fn matches_sanitized_browser_reference_fixture() {
    let fixture: Value =
        serde_json::from_str(&fs::read_to_string("tests/fixtures/browser-reference.json").unwrap())
            .unwrap();
    assert_eq!(fixture["source"]["revision"], REVISION);

    let state = fixture["prng"]["state"]
        .as_array()
        .unwrap()
        .iter()
        .map(Value::as_f64)
        .collect::<Option<Vec<_>>>()
        .unwrap();
    let mut rng = Alea::from_state(AleaState([state[0], state[1], state[2], state[3]]));
    let expected = fixture["prng"]["uint32_values"]
        .as_array()
        .unwrap()
        .iter()
        .map(Value::as_f64)
        .collect::<Option<Vec<_>>>()
        .unwrap();
    assert_eq!(
        expected,
        (0..expected.len())
            .map(|_| rng.uint32())
            .collect::<Vec<_>>()
    );

    assert_eq!(
        lfsr(
            fixture["validator"]["input"].as_str().unwrap(),
            fixture["validator"]["synthetic_salt"].as_i64().unwrap() as i32
        ),
        fixture["validator"]["result"].as_i64().unwrap() as i32
    );
}

#[test]
fn records_all_browser_report_triggers_in_browser_field_order() {
    let fixture: Value = serde_json::from_str(
        &fs::read_to_string("tests/fixtures/browser-report-trace.json").unwrap(),
    )
    .unwrap();
    assert_eq!(fixture["source"]["revision"], REVISION);
    assert_eq!(
        fixture["source"]["content_sha256"],
        "63926cda54b232c5e5be4e97123710ee4ab2dbd1faa213b21ddc01dab54a1ce0"
    );
    assert_eq!(
        fixture["field_order"],
        serde_json::json!([
            "cmd", "t", "n", "r", "c", "l", "x", "i", "z", "k", "a", "h", "rev", "m"
        ])
    );
    assert_eq!(
        fixture["events"]
            .as_array()
            .unwrap()
            .iter()
            .map(|event| event["trigger"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["s", "b", "m", "l", "a"]
    );
    assert_eq!(fixture["events"][3]["level"], "2");
    assert_eq!(fixture["events"][4]["best_plot"], "Act I");
}
