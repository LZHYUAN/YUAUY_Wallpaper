//! Setup checks for the selected dependencies, without implementing app settings.

use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use serde::{Deserialize, Serialize};

#[test]
fn yaml_round_trip_preserves_unicode_and_windows_paths() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Fixture {
        label: String,
        path: String,
    }

    let fixture = Fixture {
        label: "測試圖片".into(),
        path: r"C:\Public Pictures\測試".into(),
    };
    let yaml = yaml_serde::to_string(&fixture).expect("serialize synthetic fixture");
    let restored: Fixture = yaml_serde::from_str(&yaml).expect("deserialize fixture");
    assert_eq!(restored, fixture);
}

#[test]
fn tracked_yaml_example_is_parseable() {
    let example = include_str!("../../specs/001-rust-wallpaper-app/contracts/config.example.yaml");
    let value: yaml_serde::Value = yaml_serde::from_str(example).expect("parse example");
    let serialized = yaml_serde::to_string(&value).expect("serialize example");
    assert_eq!(
        yaml_serde::from_str::<yaml_serde::Value>(&serialized).expect("parse round trip"),
        value
    );
}

#[test]
fn seeded_rng_supports_reproducible_permutations() {
    let mut first: Vec<_> = (0..11).collect();
    let mut second = first.clone();
    first.shuffle(&mut StdRng::seed_from_u64(42));
    second.shuffle(&mut StdRng::seed_from_u64(42));
    assert_eq!(first, second);
    first.sort_unstable();
    assert_eq!(first, (0..11).collect::<Vec<_>>());
}

#[test]
fn temporary_fixtures_are_cleaned_up() {
    let path = {
        let directory = tempfile::tempdir().expect("create isolated synthetic fixture");
        let file = directory.path().join("fixture.txt");
        std::fs::write(&file, "synthetic test data").expect("write fixture");
        assert_eq!(
            std::fs::read_to_string(file).unwrap(),
            "synthetic test data"
        );
        directory.path().to_owned()
    };
    assert!(!path.exists());
}
