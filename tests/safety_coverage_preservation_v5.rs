mod support;

use soma_zero::{SafetyCoveragePreservationReportV5Status, Sprint89CandleRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn safety_coverage_v5_preserves_all_required_guards() {
    let config = sprint::sprint89_config_from_example(
        "soma_safety_coverage_preservation_v5.toml",
        "safety-v5",
    );
    let report = Sprint89CandleRecoveryRunner::default()
        .run_safety_coverage_preservation_v5(&config)
        .expect("report");
    assert_eq!(
        report.safety_status,
        SafetyCoveragePreservationReportV5Status::SafetyCoveragePreserved
    );
    assert!(report.live_trading_guard_present);
    assert!(report.broker_guard_present);
    assert!(report.runtime_llm_guard_present);
    assert!(report.committee_cli_safety_isolated);
    assert!(report.candle_source_boundary_preserved);
    assert!(report.candle_no_lookahead_preserved);
    assert!(report.candle_missing_auth_preserved);
    assert!(report.candle_storage_budget_preserved);
}
