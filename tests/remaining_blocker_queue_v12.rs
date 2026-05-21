mod support;

use soma_zero::RemainingBlockerQueueV12Status;
use support::sprint69_support as sprint;

#[test]
fn sprint96_remaining_blocker_queue_marks_counterfactual_as_next() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "remaining-blocker-queue-v12",
    );
    let report = bundle.remaining_blocker_queue_v12;
    assert_eq!(
        report.queue_status,
        RemainingBlockerQueueV12Status::QueueAdvanced
    );
    assert_eq!(report.primary_next_family, "CounterfactualBackfill");
    assert!(report.counterfactual_backfill_entry_allowed);
}
