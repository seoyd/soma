mod support;

use soma_zero::MeasuredTargetDeltaStatusV12;
use support::sprint69_support as sprint;

#[test]
fn sprint96_measured_target_delta_stays_sample_backed_only() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "measured-target-delta-v12",
    );
    let report = bundle.measured_target_delta_report_v12;
    assert_eq!(
        report.delta_status,
        MeasuredTargetDeltaStatusV12::SampleBackedOnly
    );
    assert!(!report.measured);
    assert!(report.sample_backed);
}
