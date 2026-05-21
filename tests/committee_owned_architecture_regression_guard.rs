mod support;

use soma_zero::{CommitteeOwnedArchitectureRegressionStatus, CommitteeQualityHardeningConfig};
use support::sprint99_support::run_sprint99;

#[test]
fn committee_owned_architecture_regression_guard_blocks_regression() {
    let bundle = run_sprint99(
        "soma_committee_architecture_regression_guard.toml",
        "committee-architecture-regression-guard",
    );
    let report = bundle.committee_owned_architecture_regression_guard;
    assert_eq!(
        report.regression_status,
        CommitteeOwnedArchitectureRegressionStatus::NoRegression
    );
    assert!(report.central_core_deprecated_confirmed);
    assert!(report.committee_owned_core_confirmed);
    assert!(report.runtime_deferred_confirmed);
    assert!(report.training_deferred_confirmed);
    assert!(report.live_execution_absent);
}

#[test]
fn sprint99_config_defaults_are_safe() {
    let config = CommitteeQualityHardeningConfig::default();
    assert!(config.require_committee_owned_core);
    assert!(config.require_no_central_core_leak);
    assert!(config.preserve_runtime_deferred);
    assert!(config.preserve_safety_guards);
    let text = config.to_toml_string().expect("toml");
    assert!(!text.contains("training_"));
    assert!(!text.contains("broker"));
    assert!(!text.contains("account"));
    assert!(!text.contains("order"));
}
