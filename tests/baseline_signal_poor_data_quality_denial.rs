mod support;

use soma_zero::BaselineSignalPoorDataQualityDenialStatus;
use support::sprint69_support as sprint;

#[test]
fn baseline_signal_poor_data_quality_denial_stays_preserved() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "baseline-signal-poor-data-quality-denial",
    );
    let report = bundle.baseline_signal_poor_data_quality_denial_report;
    assert_eq!(
        report.denial_status,
        BaselineSignalPoorDataQualityDenialStatus::PoorDataQualityDenialPreserved
    );
    assert!(report.missing_data_denied);
    assert!(report.invalid_data_denied);
    assert!(report.insufficient_history_denied);
    assert!(report.low_coverage_denied);
}
