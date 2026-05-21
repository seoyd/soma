mod support;

use soma_zero::SevenBlockerQueueProgressStatusV13;
use support::sprint69_support as sprint;

#[test]
fn seven_blocker_queue_progress_v13_closes_queue() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "queue-progress-v13",
    )
    .seven_blocker_queue_progress_report_v13;
    assert_eq!(
        report.queue_status,
        SevenBlockerQueueProgressStatusV13::QueueClosed
    );
    assert!(report.current_queue.is_empty());
}
