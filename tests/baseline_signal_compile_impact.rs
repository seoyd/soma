mod support;

use soma_zero::BaselineSignalCompileImpactStatus;
use support::sprint69_support as sprint;

#[test]
fn baseline_signal_compile_impact_stays_sample_backed_only() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "baseline-signal-compile-impact",
    );
    let report = bundle.baseline_signal_compile_impact_report;
    assert_eq!(
        report.impact_status,
        BaselineSignalCompileImpactStatus::CompileImpactSampleBacked
    );
    assert!(!report.measured);
    assert!(report.sample_backed);
    assert_eq!(report.baseline_signal_delta, Some(0));
}
