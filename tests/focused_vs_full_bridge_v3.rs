mod support;

use soma_zero::WorkspaceAcceptanceRecoveryV7Runner;
use support::sprint106_support::{sprint106_config_from_example, write_support_json};

#[test]
fn focused_vs_full_bridge_ignores_imported_full_pass_without_real_full_attempt() {
    let workspace_truth = write_support_json(
        "focused-vs-full-v3-imported-full-pass",
        "workspace_truth.json",
        &serde_json::json!({
            "previous_truth_status": "WorkspaceAcceptanceStillOpenV6",
            "current_truth_status": "WorkspaceAcceptanceStillOpenV6",
            "can_claim_full_acceptance": true,
            "no_run_started": true,
            "no_run_finished": true,
            "no_run_passed": true,
            "full_started": true,
            "full_finished": true,
            "full_passed": true
        }),
    );
    let mut config = sprint106_config_from_example(
        "soma_focused_vs_full_bridge_v3.toml",
        "focused-vs-full-v3-imported-full-pass-output",
    );
    config.workspace_truth_paths = Some(vec![workspace_truth]);

    let bundle = WorkspaceAcceptanceRecoveryV7Runner::default()
        .run(&config)
        .expect("run sprint106");

    assert!(!bundle.focused_vs_full_bridge_v3.can_claim_full_acceptance);
    assert!(!bundle.acceptance_truth_gate_v7.can_claim_full_acceptance);
    assert_eq!(
        bundle.acceptance_truth_gate_v7.truth_status,
        "AcceptanceOverclaimed"
    );
}
