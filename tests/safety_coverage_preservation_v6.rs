mod support;

use soma_zero::{
    SafetyCoveragePreservationReportV6Status, Sprint90ExternalPredictionRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn safety_coverage_v6_preserves_all_required_guards() {
    let config = sprint::sprint90_config_from_example(
        "soma_safety_coverage_preservation_v6.toml",
        "safety-v6",
    );
    let report = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_safety_coverage_preservation_v6(&config)
        .expect("report");
    assert_eq!(
        report.safety_status,
        SafetyCoveragePreservationReportV6Status::SafetyCoveragePreserved
    );
    assert!(report.live_trading_guard_present);
    assert!(report.broker_guard_present);
    assert!(report.runtime_llm_guard_present);
    assert!(report.committee_cli_safety_isolated);
    assert!(report.external_schema_preserved);
    assert!(report.external_model_card_preserved);
    assert!(report.external_forbidden_columns_rejected);
    assert!(report.external_runtime_deferred_preserved);
}
