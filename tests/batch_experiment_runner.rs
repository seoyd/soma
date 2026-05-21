mod common;

use soma_zero::{
    BatchExperimentRunner, ExperimentMode, ExperimentRunStatus, ExperimentVariant,
    ExperimentVariantOverrides, ReasonCode, Timeframe,
};

#[test]
fn batch_runner_skips_disabled_entries_and_runs_baseline_without_python() {
    let matrix = common::batch_matrix(
        "batch-runner",
        vec![
            common::dataset_entry("enabled", "generic_ohlcv_valid.csv", true),
            common::dataset_entry("disabled", "generic_ohlcv_valid.csv", false),
        ],
        vec![
            common::baseline_variant("baseline_5m", true),
            common::baseline_variant("disabled_variant", false),
        ],
    );

    let report = BatchExperimentRunner::default().run_matrix(&matrix);
    assert_eq!(report.run_summaries.len(), 3);
    assert!(
        report
            .run_summaries
            .iter()
            .any(|summary| summary.reason_codes.contains(&ReasonCode::DatasetDisabled))
    );
    assert!(
        report
            .run_summaries
            .iter()
            .any(|summary| summary.reason_codes.contains(&ReasonCode::VariantDisabled))
    );
    assert!(
        report
            .run_summaries
            .iter()
            .any(|summary| summary.run_key.dataset_id == "enabled"
                && summary.run_key.variant_id == "baseline_5m"
                && matches!(
                    summary.status,
                    ExperimentRunStatus::Passed | ExperimentRunStatus::Warning
                ))
    );
    assert!(
        std::path::Path::new(&matrix.dataset_bundle.output_root)
            .join(&matrix.matrix_id)
            .join("batch_summary.txt")
            .exists()
    );
}

#[test]
fn continue_on_failure_true_keeps_running_and_require_all_pass_flags_failure() {
    let failing_compare = ExperimentVariant {
        variant_id: "compare_missing".to_string(),
        mode: ExperimentMode::TrainAndCompare,
        overrides: ExperimentVariantOverrides {
            timeframe: Some(Timeframe::OneMinute),
            resample_to: Some(Timeframe::FiveMinute),
            run_python_training: Some(false),
            ..ExperimentVariantOverrides::default()
        },
        enabled: true,
        tags: vec!["compare".to_string()],
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    let mut matrix = common::batch_matrix(
        "batch-failure",
        vec![common::dataset_entry(
            "valid",
            "generic_ohlcv_valid.csv",
            true,
        )],
        vec![
            failing_compare,
            common::baseline_variant("baseline_5m", true),
        ],
    );
    matrix.require_all_pass = true;

    let report = BatchExperimentRunner::default().run_matrix(&matrix);
    assert_eq!(report.run_summaries.len(), 2);
    assert!(
        report
            .run_summaries
            .iter()
            .any(|summary| summary.run_key.variant_id == "compare_missing"
                && summary.status == ExperimentRunStatus::Failed)
    );
    assert!(
        report
            .run_summaries
            .iter()
            .any(|summary| summary.run_key.variant_id == "baseline_5m")
    );
    assert!(
        report
            .reason_codes
            .contains(&ReasonCode::MatrixRequireAllPassFailed)
    );
}

#[test]
fn bad_fixture_is_not_silently_reported_as_passed() {
    let matrix = common::batch_matrix(
        "batch-bad-data",
        vec![common::dataset_entry(
            "bad",
            "generic_ohlcv_bad_ohlc.csv",
            true,
        )],
        vec![common::baseline_variant("baseline_5m", true)],
    );

    let report = BatchExperimentRunner::default().run_matrix(&matrix);
    assert_eq!(report.run_summaries.len(), 1);
    assert!(matches!(
        report.run_summaries[0].status,
        ExperimentRunStatus::Failed | ExperimentRunStatus::Warning
    ));
}
