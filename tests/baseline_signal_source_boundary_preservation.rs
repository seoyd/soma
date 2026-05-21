mod support;

use soma_zero::BaselineSignalSourceBoundaryStatus;
use support::sprint69_support as sprint;

#[test]
fn baseline_signal_source_boundary_stays_explicit() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "baseline-signal-source-boundary-preservation",
    );
    let report = bundle.baseline_signal_source_boundary_preservation_report;
    assert_eq!(
        report.source_boundary_status,
        BaselineSignalSourceBoundaryStatus::SourceBoundaryPreserved
    );
    assert!(report.official_source_class_preserved);
    assert!(report.research_only_source_class_preserved);
    assert!(report.diagnostic_only_source_class_preserved);
    assert!(report.no_source_promotion);
}
