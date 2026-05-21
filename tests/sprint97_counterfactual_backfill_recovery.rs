mod support;

use soma_zero::{FinalBlockerQueueClosureStatus, WorkspaceAcceptanceTruthGateStatus};
use support::sprint69_support as sprint;

#[test]
fn sprint97_counterfactual_backfill_recovery_bundle_stays_consistent() {
    let bundle = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "sprint97-counterfactual-bundle",
    );
    assert_eq!(
        bundle.final_blocker_queue_closure_report.closure_status,
        FinalBlockerQueueClosureStatus::QueueClosedWithWorkspaceStillBlocked
    );
    assert_eq!(
        bundle.workspace_acceptance_truth_gate.truth_status,
        WorkspaceAcceptanceTruthGateStatus::FullWorkspaceNotRun
    );
}
