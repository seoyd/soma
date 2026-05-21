mod common;

use std::fs;

use soma_zero::{CampaignMatrixStatus, ReasonCode, ResearchCampaignRunner};

fn write_matrix(name: &str, matrix: soma_zero::ExperimentMatrixConfig) -> String {
    let path = common::output_dir(name).join("matrix.toml");
    fs::write(&path, matrix.to_toml_string().expect("matrix toml")).expect("write matrix");
    path.display().to_string()
}

#[test]
fn campaign_runner_runs_valid_batch_matrix_and_baseline_needs_no_python() {
    let matrix_path = write_matrix(
        "campaign-valid",
        common::batch_matrix(
            "campaign-valid-matrix",
            vec![common::dataset_entry(
                "valid",
                "generic_ohlcv_valid.csv",
                true,
            )],
            vec![common::baseline_variant("baseline_5m", true)],
        ),
    );
    let config = common::campaign_config("campaign-valid", vec![matrix_path]);
    let report = ResearchCampaignRunner::default().run_campaign(&config);
    assert_eq!(report.matrix_results.len(), 1);
    assert_eq!(
        report.matrix_results[0].status,
        CampaignMatrixStatus::Passed
    );
    assert!(report.aggregate.total_runs >= 1);
    assert!(report.errors.is_empty());
}

#[test]
fn campaign_runner_runs_multiple_matrices_in_deterministic_order() {
    let first = write_matrix(
        "campaign-order-b",
        common::batch_matrix(
            "b_matrix",
            vec![common::dataset_entry(
                "valid_b",
                "generic_ohlcv_valid.csv",
                true,
            )],
            vec![common::baseline_variant("baseline_5m", true)],
        ),
    );
    let second = write_matrix(
        "campaign-order-a",
        common::batch_matrix(
            "a_matrix",
            vec![common::dataset_entry(
                "valid_a",
                "generic_ohlcv_valid.csv",
                true,
            )],
            vec![common::baseline_variant("baseline_5m", true)],
        ),
    );
    let config = common::campaign_config("campaign-order", vec![first, second]);
    let report = ResearchCampaignRunner::default().run_campaign(&config);
    assert_eq!(report.matrix_results[0].matrix_id, "a_matrix");
    assert_eq!(report.matrix_results[1].matrix_id, "b_matrix");
}

#[test]
fn campaign_runner_records_failed_matrix_and_can_require_all_pass() {
    let valid = write_matrix(
        "campaign-failed-valid",
        common::batch_matrix(
            "valid_matrix",
            vec![common::dataset_entry(
                "valid",
                "generic_ohlcv_valid.csv",
                true,
            )],
            vec![common::baseline_variant("baseline_5m", true)],
        ),
    );
    let invalid = common::output_dir("campaign-failed-invalid")
        .join("missing.toml")
        .display()
        .to_string();
    let mut config = common::campaign_config("campaign-failed", vec![valid, invalid]);
    config.require_all_matrices_pass = true;

    let report = ResearchCampaignRunner::default().run_campaign(&config);
    assert!(report.matrix_results.iter().any(|result| {
        result
            .reason_codes
            .contains(&ReasonCode::CampaignMatrixLoadFailed)
    }));
    assert!(
        report
            .reason_codes
            .contains(&ReasonCode::CampaignRequireAllPassFailed)
    );
}
