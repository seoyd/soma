mod common;
#[path = "support/sprint67_support.rs"]
mod sprint67_support;

use soma_zero::{
    ModelOpsRegressionGuardReport, ModelOpsRegressionGuardStatus, ModelOpsRollupRunner,
    RegressionCauseExplanationStatus, RegressionCauseKind,
};

#[test]
fn regression_explanations_cover_all_expected_regression_kinds() {
    let config = sprint67_support::rollup_config_from_example(
        "soma_model_regression_explain.toml",
        "regression-explanations",
    );
    let report = ModelOpsRollupRunner::default()
        .run_regression_explain(&config)
        .expect("run regression explain");
    assert_eq!(
        report.report_status,
        RegressionCauseExplanationStatus::RegressionDetected
    );
    for cause in [
        RegressionCauseKind::CoverageRegression,
        RegressionCauseKind::CalibrationRegression,
        RegressionCauseKind::RiskRegression,
        RegressionCauseKind::ComparabilityRegression,
        RegressionCauseKind::ArtifactCompletenessRegression,
        RegressionCauseKind::LeaderboardRegression,
    ] {
        assert!(
            report
                .explanations
                .iter()
                .any(|item| item.cause_kind == cause)
        );
    }
}

#[test]
fn regression_explainer_handles_missing_baseline_and_no_regression() {
    let mut config = sprint67_support::rollup_config_from_example(
        "soma_model_regression_explain.toml",
        "regression-missing-baseline",
    );
    let mut missing: ModelOpsRegressionGuardReport =
        sprint67_support::read_json(&config.model_ops_regression_guard_paths[0]);
    missing.coverage_regressions.clear();
    missing.calibration_regressions.clear();
    missing.risk_regressions.clear();
    missing.comparability_regressions.clear();
    missing.artifact_completeness_regressions.clear();
    missing.leaderboard_regressions.clear();
    missing.guard_status = ModelOpsRegressionGuardStatus::MissingBaseline;
    config.model_ops_regression_guard_paths[0] =
        sprint67_support::write_support_json("regression-missing-baseline", "guard.json", &missing);
    let report = ModelOpsRollupRunner::default()
        .run_regression_explain(&config)
        .expect("run missing-baseline explain");
    assert_eq!(
        report.report_status,
        RegressionCauseExplanationStatus::NeedsBaseline
    );
    assert!(
        report
            .explanations
            .iter()
            .any(|item| item.cause_kind == RegressionCauseKind::MissingBaseline)
    );

    let mut config = sprint67_support::rollup_config_from_example(
        "soma_model_regression_explain.toml",
        "regression-none",
    );
    let mut none: ModelOpsRegressionGuardReport =
        sprint67_support::read_json(&config.model_ops_regression_guard_paths[0]);
    none.coverage_regressions.clear();
    none.calibration_regressions.clear();
    none.risk_regressions.clear();
    none.comparability_regressions.clear();
    none.artifact_completeness_regressions.clear();
    none.leaderboard_regressions.clear();
    none.guard_status = ModelOpsRegressionGuardStatus::NoRegression;
    config.model_ops_regression_guard_paths[0] =
        sprint67_support::write_support_json("regression-none", "guard.json", &none);
    let report = ModelOpsRollupRunner::default()
        .run_regression_explain(&config)
        .expect("run no-regression explain");
    assert_eq!(
        report.report_status,
        RegressionCauseExplanationStatus::NoRegression
    );
    assert!(
        report
            .explanations
            .iter()
            .all(|item| item.cause_kind == RegressionCauseKind::NoRegression)
    );
}
