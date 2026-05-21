mod support;

use soma_zero::{SafetyCoveragePreservationReportV2Status, Sprint86ResidualGateRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn safety_coverage_preservation_v2_keeps_all_guard_flags() {
    let config = sprint::sprint86_config_from_example(
        "soma_safety_coverage_preservation_v2.toml",
        "safety-coverage-preservation-v2-test",
    );
    let report = Sprint86ResidualGateRecoveryRunner::default()
        .run_safety_coverage_preservation_v2(&config)
        .expect("safety");
    assert_eq!(
        report.safety_status,
        SafetyCoveragePreservationReportV2Status::SafetyCoveragePreserved
    );
    assert!(report.live_trading_guard_present);
    assert!(report.browser_execution_guard_present);
    assert!(report.no_lookahead_guard_present);
    assert!(report.source_boundary_guard_present);
}
