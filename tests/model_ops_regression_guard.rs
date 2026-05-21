mod common;
#[path = "support/sprint66_support.rs"]
mod sprint66_support;

use soma_zero::{
    ModelOpsRegressionGuardStatus, ModelOpsRegressionSnapshot, ModelReviewClosureRunner,
};

#[test]
fn regression_guard_detects_drops_and_statuses_regression_detected() {
    let config = sprint66_support::closure_config_from_example(
        "soma_model_ops_regression_guard.toml",
        "model-ops-regression-guard",
    );
    let report = ModelReviewClosureRunner::default()
        .run_regression_guard(&config)
        .expect("run regression guard");
    assert_eq!(
        report.guard_status,
        ModelOpsRegressionGuardStatus::RegressionDetected
    );
    assert!(!report.coverage_regressions.is_empty());
    assert!(!report.calibration_regressions.is_empty());
    assert!(!report.risk_regressions.is_empty());
    assert!(!report.comparability_regressions.is_empty());
    assert!(!report.artifact_completeness_regressions.is_empty());
    assert!(!report.leaderboard_regressions.is_empty());
}

#[test]
fn regression_guard_passes_when_current_matches_baseline() {
    let mut config = sprint66_support::closure_config_from_example(
        "soma_model_ops_regression_guard.toml",
        "model-ops-regression-guard-pass",
    );
    config.model_ops_current_paths = config.model_ops_baseline_paths.clone();
    let report = ModelReviewClosureRunner::default()
        .run_regression_guard(&config)
        .expect("run passing regression guard");
    assert_eq!(
        report.guard_status,
        ModelOpsRegressionGuardStatus::NoRegression
    );
    assert!(report.coverage_regressions.is_empty());
    assert!(report.calibration_regressions.is_empty());
    assert!(report.risk_regressions.is_empty());
    assert!(report.comparability_regressions.is_empty());
    assert!(report.artifact_completeness_regressions.is_empty());
    assert!(report.leaderboard_regressions.is_empty());
}

#[test]
fn regression_guard_keeps_missing_baseline_explicit() {
    let mut config = sprint66_support::closure_config_from_example(
        "soma_model_ops_regression_guard.toml",
        "model-ops-regression-guard-missing-baseline",
    );
    config.model_ops_baseline_paths.clear();
    let report = ModelReviewClosureRunner::default()
        .run_regression_guard(&config)
        .expect("run missing-baseline regression guard");
    assert_eq!(
        report.guard_status,
        ModelOpsRegressionGuardStatus::MissingBaseline
    );
}

#[test]
fn regression_guard_report_is_deterministic() {
    let config = sprint66_support::closure_config_from_example(
        "soma_model_ops_regression_guard.toml",
        "model-ops-regression-guard-deterministic",
    );
    let runner = ModelReviewClosureRunner::default();
    let first = runner
        .run_regression_guard(&config)
        .expect("first regression guard");
    let second = runner
        .run_regression_guard(&config)
        .expect("second regression guard");
    assert_eq!(first, second);
}

#[test]
fn regression_snapshot_json_round_trip_is_stable() {
    let path = sprint66_support::example_path("sprint66_data/model_ops_current.json");
    let snapshot: ModelOpsRegressionSnapshot = sprint66_support::read_json(&path);
    let encoded = serde_json::to_string_pretty(&snapshot).expect("encode regression snapshot");
    let decoded: ModelOpsRegressionSnapshot =
        serde_json::from_str(&encoded).expect("decode regression snapshot");
    assert_eq!(snapshot, decoded);
}
