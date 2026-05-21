mod common;

use std::fs;
use std::path::PathBuf;

use soma_zero::OfficialEvidenceAcquisitionPlan;

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

#[test]
fn official_acquisition_examples_parse() {
    for path in [
        example_path("soma_official_evidence_acquisition.toml"),
        example_path("soma_official_evidence_acquisition_crypto_only.toml"),
        example_path("soma_official_evidence_acquisition_multi_venue.toml"),
    ] {
        let plan = OfficialEvidenceAcquisitionPlan::from_toml_path(&path).expect("parse");
        assert!(!plan.plan_id.is_empty());
    }
}

#[test]
fn official_acquisition_runner_output_is_deterministic_for_same_config() {
    let config_path = common::output_dir("official-acquisition-determinism").join("acquire.toml");
    fs::write(
        &config_path,
        OfficialEvidenceAcquisitionPlan {
            run_collection: false,
            expansion_config: None,
            ..OfficialEvidenceAcquisitionPlan::default()
        }
        .to_toml_string()
        .expect("serialize"),
    )
    .expect("write config");
    let first = OfficialEvidenceAcquisitionPlan::from_toml_path(&config_path)
        .expect("first parse")
        .to_toml_string()
        .expect("first toml");
    let second = OfficialEvidenceAcquisitionPlan::from_toml_path(&config_path)
        .expect("second parse")
        .to_toml_string()
        .expect("second toml");

    assert_eq!(first, second);
}
