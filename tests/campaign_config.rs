mod common;

use std::fs;

use soma_zero::{ReasonCode, ResearchCampaignConfig, ResearchCampaignRunner};

#[test]
fn campaign_config_can_be_constructed_with_conservative_defaults() {
    let config = ResearchCampaignConfig::default();
    assert_eq!(config.min_usable_datasets, 2);
    assert_eq!(config.min_total_outcome_records, 20);
    assert_eq!(config.min_regime_coverage_count, 2);
    assert_eq!(config.min_passed_runs, 2);
    assert!(!config.allow_persona_expansion_recommendation);
}

#[test]
fn campaign_config_rejects_remote_matrix_paths() {
    let config = ResearchCampaignConfig {
        matrix_config_paths: vec!["https://example.com/matrix.toml".to_string()],
        ..ResearchCampaignConfig::default()
    };
    let reasons = config.validate_local_paths();
    assert!(reasons.contains(&ReasonCode::LocalPathRejected));
    assert!(reasons.contains(&ReasonCode::CampaignConfigInvalid));
}

#[test]
fn missing_compare_target_marks_diff_unavailable_not_failure() {
    let matrix = common::batch_matrix(
        "campaign-config-matrix",
        vec![common::dataset_entry(
            "valid",
            "generic_ohlcv_valid.csv",
            true,
        )],
        vec![common::baseline_variant("baseline_5m", true)],
    );
    let matrix_path = common::output_dir("campaign-config-matrix").join("matrix.toml");
    fs::write(&matrix_path, matrix.to_toml_string().expect("matrix toml")).expect("write matrix");

    let config =
        common::campaign_config("campaign-config", vec![matrix_path.display().to_string()]);
    let report = ResearchCampaignRunner::default().run_campaign(&config);
    assert!(!report.diff_report.comparable);
    assert!(
        report
            .diff_report
            .reason_codes
            .contains(&ReasonCode::CampaignDiffUnavailable)
    );
    assert!(report.errors.is_empty());
}
