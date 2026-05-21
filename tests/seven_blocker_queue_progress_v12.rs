mod support;

use soma_zero::SevenBlockerQueueProgressStatusV12;
use support::sprint69_support as sprint;

#[test]
fn sprint96_queue_progress_advances_to_counterfactual_backfill() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "seven-blocker-queue-progress-v12",
    );
    let report = bundle.seven_blocker_queue_progress_report_v12;
    assert_eq!(
        report.queue_status,
        SevenBlockerQueueProgressStatusV12::QueueAdvanced
    );
    assert_eq!(report.primary_next_family, "CounterfactualBackfill");
    assert_eq!(report.current_queue, vec!["CounterfactualBackfill"]);
}
