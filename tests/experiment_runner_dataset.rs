mod common;

use std::fs;

use soma_zero::{ExperimentMode, ExperimentRunner, ExperimentStage, StageStatus};

#[test]
fn dataset_export_only_writes_dataset_summary_and_csv() {
    let config = common::dataset_config("dataset-run", "generic_ohlcv_valid.csv");
    let bundle = ExperimentRunner::default().run(&config);
    assert!(bundle.dataset_export_summary.is_some());
    assert_eq!(
        bundle.stage_status(ExperimentStage::BuildDataset),
        StageStatus::Passed
    );
    assert!(config.output_bundle_dir().join("dataset.csv").exists());
}

#[test]
fn dataset_export_uses_stable_feature_order() {
    let config = common::dataset_config("dataset-order", "generic_ohlcv_valid.csv");
    let first = ExperimentRunner::default().run(&config);
    let first_csv =
        fs::read_to_string(config.output_bundle_dir().join("dataset.csv")).expect("dataset");
    let second = ExperimentRunner::default().run(&config);
    let second_csv =
        fs::read_to_string(config.output_bundle_dir().join("dataset.csv")).expect("dataset");
    assert_eq!(first.dataset_export_summary, second.dataset_export_summary);
    assert_eq!(first_csv, second_csv);
}

#[test]
fn dataset_export_includes_labels_only_in_label_columns_and_has_no_nan_or_inf() {
    let config = common::dataset_config("dataset-labels", "generic_ohlcv_valid.csv");
    let bundle = ExperimentRunner::default().run(&config);
    let csv = fs::read_to_string(config.output_bundle_dir().join("dataset.csv")).expect("dataset");
    let header = csv.lines().next().expect("header");
    assert!(header.contains("label_outcome"));
    assert!(
        !bundle
            .dataset_export_summary
            .expect("summary")
            .feature_names
            .iter()
            .any(|name| name.contains("label"))
    );
    assert!(!csv.contains("NaN"));
    assert!(!csv.contains("inf"));
}

#[test]
fn validate_data_only_returns_data_quality_report_bundle() {
    let mut config = common::baseline_config("validate-only", "generic_ohlcv_valid.csv");
    config.mode = ExperimentMode::ValidateDataOnly;
    let bundle = ExperimentRunner::default().run(&config);
    assert_eq!(
        bundle.stage_status(ExperimentStage::ValidateData),
        StageStatus::Passed
    );
    assert!(
        config
            .output_bundle_dir()
            .join("data_quality_report.txt")
            .exists()
    );
}
