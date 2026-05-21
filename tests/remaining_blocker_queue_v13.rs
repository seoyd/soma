mod support;

use soma_zero::RemainingBlockerQueueV13Status;
use support::sprint69_support as sprint;

#[test]
fn remaining_blocker_queue_v13_is_closed() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "remaining-queue-v13",
    )
    .remaining_blocker_queue_v13;
    assert_eq!(
        report.queue_status,
        RemainingBlockerQueueV13Status::QueueClosedWithWorkspaceStillBlocked
    );
    assert!(report.ordered_remaining_families.is_empty());
}
