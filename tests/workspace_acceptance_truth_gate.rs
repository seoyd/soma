mod support;

use soma_zero::WorkspaceAcceptanceTruthGateStatus;
use support::sprint69_support as sprint;

#[test]
fn workspace_acceptance_truth_gate_stays_conservative() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "workspace-acceptance-truth",
    )
    .workspace_acceptance_truth_gate;
    assert_eq!(
        report.truth_status,
        WorkspaceAcceptanceTruthGateStatus::FullWorkspaceNotRun
    );
    assert!(!report.can_claim_full_acceptance);
}
