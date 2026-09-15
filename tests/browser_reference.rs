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
