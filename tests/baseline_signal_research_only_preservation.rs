mod support;

use soma_zero::BaselineSignalResearchOnlyStatus;
use support::sprint69_support as sprint;

#[test]
fn baseline_signal_research_only_interpretation_stays_preserved() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "baseline-signal-research-only-preservation",
    );
    let report = bundle.baseline_signal_research_only_preservation_report;
    assert_eq!(
        report.research_status,
        BaselineSignalResearchOnlyStatus::ResearchOnlyPreserved
    );
    assert!(report.offline_only_preserved);
    assert!(report.research_only_preserved);
    assert!(report.paper_only_preserved);
    assert!(report.no_live_signal_activation);
}
