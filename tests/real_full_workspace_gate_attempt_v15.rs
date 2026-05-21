mod support;

use soma_zero::RealFullWorkspaceGateAttemptV15Status;
use support::sprint69_support as sprint;

#[test]
fn real_full_workspace_gate_attempt_v15_stays_not_run_in_smoke_fixture() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "real-full-v15",
    )
    .real_full_workspace_gate_attempt_v15;
    assert_eq!(
        report.full_status,
        RealFullWorkspaceGateAttemptV15Status::NotRun
    );
}
