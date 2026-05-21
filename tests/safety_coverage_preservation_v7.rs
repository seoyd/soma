mod support;

use soma_zero::{SafetyCoveragePreservationReportV7Status, Sprint91KrxEvidenceRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn safety_coverage_v7_preserves_all_required_guards() {
    let config = sprint::sprint91_config_from_example(
        "soma_safety_coverage_preservation_v7.toml",
        "krx-safety-v7",
    );
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_safety_coverage_preservation_v7(&config)
        .expect("report");
    assert_eq!(
        report.safety_status,
        SafetyCoveragePreservationReportV7Status::SafetyCoveragePreserved
    );
    assert!(report.live_trading_guard_present);
    assert!(report.broker_guard_present);
    assert!(report.runtime_llm_guard_present);
    assert!(report.committee_cli_safety_isolated);
    assert!(report.krx_market_data_only_preserved);
    assert!(report.krx_missing_auth_preserved);
    assert!(report.krx_endpoint_template_preserved);
    assert!(report.krx_source_boundary_preserved);
    assert!(report.krx_no_order_account_preserved);
}
