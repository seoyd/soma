mod support;

use soma_zero::SafeConsolidationPatchV2Runner;
use support::sprint108_support::{run_sprint108, sprint108_config_from_example};

#[test]
fn workspace_no_run_recovery_gate_stays_honest_by_default() {
    let bundle = run_sprint108(
        "soma_workspace_no_run_recovery_gate_v9.toml",
        "workspace-no-run-recovery-gate-v9",
    );
    assert_eq!(
        bundle.workspace_no_run_recovery_gate_v9.gate_status,
        "NoRunNotRun"
    );
    assert!(!bundle.workspace_no_run_recovery_gate_v9.no_run_recovered);
    assert_eq!(
        bundle.workspace_full_acceptance_gate_v9.gate_status,
        "FullWorkspaceNotRun"
    );
    assert!(
        !bundle
            .workspace_full_acceptance_gate_v9
            .full_workspace_accepted
    );
}

#[test]
fn workspace_timeout_cannot_claim_no_run_or_full_acceptance() {
    let mut config = sprint108_config_from_example(
        "soma_workspace_no_run_recovery_gate_v9.toml",
        "workspace-no-run-recovery-gate-v9-timeout",
    );
    config.run_real_no_run_after_patch = true;
    config.no_run_timeout_ms = Some(1);
    config.run_real_full_after_patch = true;
    config.full_timeout_ms = Some(1);
    let bundle = SafeConsolidationPatchV2Runner::default()
        .run(&config)
        .expect("run");
    assert!(!bundle.workspace_no_run_recovery_gate_v9.no_run_recovered);
    assert!(
        !bundle
            .workspace_full_acceptance_gate_v9
            .full_workspace_accepted
    );
    assert_ne!(
        bundle.workspace_no_run_recovery_gate_v9.gate_status,
        "NoRunRecovered"
    );
    assert_ne!(
        bundle.workspace_full_acceptance_gate_v9.gate_status,
        "FullWorkspaceAccepted"
    );
}
