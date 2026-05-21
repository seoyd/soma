mod support;

use soma_zero::{FinalBlockerQueueClosureGateStatus, FinalBlockerQueueClosureStatus};
use support::sprint69_support as sprint;

#[test]
fn final_blocker_queue_closure_is_explicit() {
    let bundle = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "final-blocker-queue-closure",
    );
    assert_eq!(
        bundle.final_blocker_queue_closure_gate.gate_status,
        FinalBlockerQueueClosureGateStatus::FinalBlockerQueueClosedWithWorkspaceStillBlocked
    );
    assert_eq!(
        bundle.final_blocker_queue_closure_report.closure_status,
        FinalBlockerQueueClosureStatus::QueueClosedWithWorkspaceStillBlocked
    );
    assert!(
        bundle
            .final_blocker_queue_closure_report
            .remaining_families
            .is_empty()
    );
}
