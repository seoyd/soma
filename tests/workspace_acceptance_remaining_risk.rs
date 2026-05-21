mod support;

use soma_zero::WorkspaceAcceptanceRemainingRiskStatus;
use support::sprint69_support as sprint;

#[test]
fn workspace_acceptance_remaining_risk_stays_explicit() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "workspace-remaining-risk",
    )
    .workspace_acceptance_remaining_risk_report;
    assert_eq!(
        report.risk_status,
        WorkspaceAcceptanceRemainingRiskStatus::AcceptanceRiskRemaining
    );
    assert!(!report.remaining_risks.is_empty());
}
