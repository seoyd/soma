mod support;

use soma_zero::BaselineSignalNoLookaheadStatus;
use support::sprint69_support as sprint;

#[test]
fn baseline_signal_no_lookahead_checks_stay_preserved() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "baseline-signal-no-lookahead-preservation",
    );
    let report = bundle.baseline_signal_no_lookahead_preservation_report;
    assert_eq!(
        report.no_lookahead_status,
        BaselineSignalNoLookaheadStatus::NoLookaheadPreserved
    );
    assert!(report.future_outcome_not_used_as_feature);
    assert!(report.future_bar_not_used_before_timestamp);
    assert!(report.label_not_used_for_signal);
    assert!(report.sequence_horizon_safe);
}
