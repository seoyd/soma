mod common;

use std::path::PathBuf;

use soma_zero::{CandleCsvLoader, DataQualitySeverity, EvidenceClosureConfig, ReasonCode};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn evidence_closure_config_defaults_match_sprint15_targets() {
    let config = EvidenceClosureConfig::default();
    assert_eq!(config.min_additional_usable_datasets, 1);
    assert_eq!(config.min_additional_outcome_records, 20);
    assert_eq!(config.min_additional_comparable_variants, 2);
    assert!(config.allow_synthetic_dataset);
    assert!(config.synthetic_dataset_must_be_tagged);
    assert!(config.strict_data_quality);
}

#[test]
fn evidence_closure_config_rejects_remote_paths() {
    let config = EvidenceClosureConfig {
        output_root: "https://remote.invalid/out".to_string(),
        ..EvidenceClosureConfig::default()
    };
    assert!(
        config
            .validate_local_paths()
            .contains(&ReasonCode::LocalPathRejected)
    );
}

#[test]
fn new_alt_fixture_loads_and_has_usable_quality() {
    let config = common::baseline_config("sprint15-alt-quality", "generic_ohlcv_valid_alt.csv");
    let loader = CandleCsvLoader::default();
    let loaded = loader
        .load_from_path(
            &repo_root()
                .join("tests")
                .join("fixtures")
                .join("market_data")
                .join("generic_ohlcv_valid_alt.csv"),
            &config.build_csv_config(),
        )
        .expect("load alt fixture");
    assert!(loaded.series.len() >= 20);
    let bundle = soma_zero::ExperimentRunner::default().run(&config);
    assert!(matches!(
        bundle.data_quality_report.severity,
        DataQualitySeverity::Good | DataQualitySeverity::Warning
    ));
}
