mod support;

use soma_zero::{
    ExternalPredictionFixtureSetupReductionStatus, Sprint90ExternalPredictionRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn external_prediction_fixture_setup_reduction_uses_shared_harness() {
    let config = sprint::sprint90_config_from_example(
        "soma_external_prediction_fixture_setup_reduction.toml",
        "external-fixture-setup",
    );
    let report = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_external_prediction_fixture_setup_reduction(&config)
        .expect("report");
    assert_eq!(
        report.reduction_status,
        ExternalPredictionFixtureSetupReductionStatus::FixtureSetupReduced
    );
    assert_eq!(report.duplicate_json_loads_removed, 2);
    assert_eq!(report.duplicate_csv_loads_removed, 1);
    assert_eq!(report.duplicate_toml_loads_removed, 1);
    assert_eq!(report.duplicate_output_dirs_removed, 1);
    assert!(report.shared_fixture_harness_used);
    assert!(report.deterministic_output_preserved);
}
