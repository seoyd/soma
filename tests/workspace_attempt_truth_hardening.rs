mod support;

use soma_zero::Sprint105VerificationPatchClosureRunner;
use support::sprint105_support::run_sprint105;
use support::sprint105_support::write_support_json;

#[test]
fn workspace_truth_hardening_counts_unrun_and_long_compile() {
    let bundle = run_sprint105(
        "soma_workspace_attempt_truth_hardening.toml",
        "workspace_attempt_truth_hardening",
    );
    let report = &bundle.workspace_attempt_truth_hardening_report;
    assert!(
        report.truth_hardening_status.contains("Hardened")
            || report.truth_hardening_status.contains("Closed")
    );
    assert!(!report.can_claim_full_acceptance);
}

#[test]
fn workspace_truth_hardening_refuses_unfinished_acceptance_claims() {
    let workspace_truth = write_support_json(
        "workspace_truth_hardening_overclaim",
        "workspace_truth.json",
        &serde_json::json!({
            "truth_status": "WorkspaceTruthImported",
            "no_run_started": true,
            "no_run_finished": true,
            "no_run_passed": true,
            "full_started": true,
            "full_finished": false,
            "can_claim_full_acceptance": true
        }),
    );
    let mut config = soma_zero::Sprint105VerificationPatchClosureConfig::default();
    config.workspace_truth_paths = Some(vec![workspace_truth]);
    let bundle = Sprint105VerificationPatchClosureRunner::default()
        .run(&config)
        .expect("run");
    assert!(
        !bundle
            .workspace_attempt_truth_hardening_report
            .can_claim_full_acceptance
    );
    assert!(
        !bundle
            .workspace_acceptance_truth_recovery_plan_v6
            .can_claim_full_acceptance
    );
    assert!(bundle.focused_vs_full_gate_bridge_v2.full_workspace_open);
}
