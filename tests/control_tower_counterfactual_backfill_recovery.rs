mod support;

use soma_zero::FinalBlockerQueueClosureStatus;
use support::sprint69_support as sprint;

#[test]
fn control_tower_counterfactual_backfill_recovery_panel_stays_read_only() {
    let panel = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "control-tower-counterfactual",
    )
    .control_tower_counterfactual_backfill_recovery_panel;
    assert_eq!(
        panel.final_queue_closure_status,
        FinalBlockerQueueClosureStatus::QueueClosedWithWorkspaceStillBlocked
    );
    assert_eq!(panel.runtime_deferred_summary, "ResearchOnly");
}
