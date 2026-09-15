use std::fs;

use gyrognome::fixtures::validate_fixture;

#[test]
fn committed_reference_fixtures_are_safe() {
    for entry in fs::read_dir("tests/fixtures").unwrap() {
        let path = entry.unwrap().path();
        validate_fixture(&path, &fs::read_to_string(&path).unwrap()).unwrap();
    }
}
