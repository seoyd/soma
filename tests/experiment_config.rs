mod common;

use std::fs;

use soma_zero::{ExperimentConfig, ExperimentMode, ReasonCode};

#[test]
fn baseline_only_config_can_be_constructed() {
    let config = common::baseline_config("baseline-config", "generic_ohlcv_valid.csv");
    assert_eq!(config.mode, ExperimentMode::BaselineOnly);
}

#[test]
fn dataset_export_only_config_can_be_constructed() {
    let config = common::dataset_config("dataset-config", "generic_ohlcv_valid.csv");
    assert_eq!(config.mode, ExperimentMode::DatasetExportOnly);
}

#[test]
fn train_and_compare_config_can_be_constructed_without_live_fields() {
    let mut config = common::baseline_config("train-compare-config", "generic_ohlcv_valid.csv");
    config.mode = ExperimentMode::TrainAndCompare;
    config.run_python_training = false;
    assert_eq!(config.mode, ExperimentMode::TrainAndCompare);
    assert!(config.validate_local_paths().is_empty());
}

#[test]
fn local_path_validation_rejects_remote_url_like_strings() {
    let mut config = common::baseline_config("remote-config", "generic_ohlcv_valid.csv");
    config.data_path = "https://example.com/data.csv".to_string();
    assert_eq!(
        config.validate_local_paths(),
        vec![
            ReasonCode::LocalPathRejected,
            ReasonCode::ExperimentConfigInvalid
        ]
    );
}

#[test]
fn example_toml_parses_into_experiment_config() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("soma_experiment_baseline.toml");
    let config = ExperimentConfig::from_toml_path(&path).expect("parse example");
    assert_eq!(config.mode, ExperimentMode::BaselineOnly);
    let serialized = config.to_toml_string().expect("serialize config");
    assert!(serialized.contains("experiment_id = \"baseline_example\""));
    let _ = fs::read_to_string(path).expect("example exists");
}
