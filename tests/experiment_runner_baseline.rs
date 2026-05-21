mod common;

use soma_zero::{ExperimentRunner, ExperimentStage, StageStatus};

#[test]
fn valid_csv_fixture_runs_baseline_only() {
    let config = common::baseline_config("baseline-run", "generic_ohlcv_valid.csv");
    let bundle = ExperimentRunner::default().run(&config);
    assert!(bundle.baseline_walk_forward_report.is_some());
    assert_eq!(
        bundle.stage_status(ExperimentStage::BaselineEvaluate),
        StageStatus::Passed
    );
}

#[test]
fn baseline_only_writes_report_bundle_and_requires_no_python() {
    let config = common::baseline_config("baseline-bundle", "generic_ohlcv_valid.csv");
    let bundle = ExperimentRunner::default().run(&config);
    let out = config.output_bundle_dir();
    assert!(out.join("manifest.txt").exists());
    assert!(out.join("baseline_report.txt").exists());
    assert_eq!(
        bundle.stage_status(ExperimentStage::PythonTrain),
        StageStatus::Skipped
    );
}

#[test]
fn baseline_only_preserves_risk_governor_veto() {
    let mut config = common::baseline_config("baseline-risk", "generic_ohlcv_valid.csv");
    config.risk_config.min_confidence = 1.1;
    config.risk_config.min_expected_edge = 10.0;
    config.risk_config.max_spread_bps = 0.0;
    let bundle = ExperimentRunner::default().run(&config);
    if let Some(report) = bundle.baseline_walk_forward_report {
        assert!(
            report.aggregate_metrics.risk_metrics.denied_count > 0
                || report.aggregate_metrics.decision_metrics.no_trade > 0
        );
    } else {
        assert_eq!(
            bundle.stage_status(ExperimentStage::ValidateData),
            StageStatus::Failed
        );
    }
}
