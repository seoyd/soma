mod support;

use soma_zero::SafetyCoveragePreservationReportV12Status;
use support::sprint69_support as sprint;

#[test]
fn sprint96_safety_coverage_stays_preserved() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "safety-coverage-preservation-v12",
    );
    let report = bundle.safety_coverage_preservation_report_v12;
    assert_eq!(
        report.safety_status,
        SafetyCoveragePreservationReportV12Status::SafetyCoveragePreserved
    );
    assert!(report.committee_cli_safety_isolated);
    assert!(report.baseline_no_trade_default_preserved);
    assert!(report.baseline_risk_governor_veto_preserved);
    assert!(report.baseline_poor_data_quality_denial_preserved);
}
