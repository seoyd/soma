mod support;

use soma_zero::BaselineSignalEntryConsumedStatus;
use support::sprint69_support as sprint;

#[test]
fn baseline_signal_entry_is_explicitly_consumed() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "baseline-signal-entry-consumed",
    );
    let report = bundle.baseline_signal_entry_consumed_report;
    assert_eq!(
        report.consumed_status,
        BaselineSignalEntryConsumedStatus::EntryConsumedForBaselineSignal
    );
    assert!(report.reduction_started);
    assert!(report.entry_consumed);
}
