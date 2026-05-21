mod common;

use soma_zero::{ExperimentMode, ReasonCode, Timeframe};

#[test]
fn dataset_bundle_and_matrix_can_be_constructed_deterministically() {
    let matrix = common::batch_matrix(
        "batch-config",
        vec![
            common::dataset_entry("dataset_a", "generic_ohlcv_valid.csv", true),
            common::dataset_entry("dataset_b", "generic_ohlcv_gaps.csv", true),
        ],
        vec![common::baseline_variant("baseline_5m", true)],
    );

    assert_eq!(matrix.dataset_bundle.entries.len(), 2);
    assert_eq!(matrix.variants.len(), 1);
    assert_eq!(matrix.dataset_bundle.entries[0].dataset_id, "dataset_a");
    assert_eq!(matrix.dataset_bundle.entries[1].dataset_id, "dataset_b");

    let config =
        matrix.build_experiment_config(&matrix.dataset_bundle.entries[0], &matrix.variants[0]);
    assert_eq!(config.mode, ExperimentMode::BaselineOnly);
    assert_eq!(config.timeframe, Timeframe::OneMinute);
    assert_eq!(config.resample_to, Some(Timeframe::FiveMinute));
    assert_eq!(
        config.experiment_id,
        "batch-config-seed-dataset_a-baseline_5m"
    );
}

#[test]
fn matrix_toml_round_trip_preserves_order() {
    let matrix = common::batch_matrix(
        "batch-roundtrip",
        vec![
            common::dataset_entry("first", "generic_ohlcv_valid.csv", true),
            common::dataset_entry("second", "generic_ohlcv_gaps.csv", true),
        ],
        vec![
            common::baseline_variant("baseline_5m", true),
            common::baseline_variant("baseline_alt", false),
        ],
    );

    let toml = matrix.to_toml_string().expect("toml");
    let decoded: soma_zero::ExperimentMatrixConfig = toml::from_str(&toml).expect("decode");

    assert_eq!(decoded.dataset_bundle.entries[0].dataset_id, "first");
    assert_eq!(decoded.dataset_bundle.entries[1].dataset_id, "second");
    assert_eq!(decoded.variants[0].variant_id, "baseline_5m");
    assert_eq!(decoded.variants[1].variant_id, "baseline_alt");
}

#[test]
fn remote_url_like_paths_are_rejected() {
    let mut entry = common::dataset_entry("remote", "generic_ohlcv_valid.csv", true);
    entry.data_path = "https://example.com/data.csv".to_string();
    let matrix = common::batch_matrix(
        "batch-remote",
        vec![entry],
        vec![common::baseline_variant("baseline_5m", true)],
    );

    let reasons = matrix.validate_local_paths();
    assert!(reasons.contains(&ReasonCode::LocalPathRejected));
    assert!(reasons.contains(&ReasonCode::ExperimentConfigInvalid));
}
