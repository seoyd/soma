mod support;

use serde_json::to_value;
use soma_zero::{DualAgentWorkflowConfig, Sprint104DualAgentPaperLifecycleRunner};
use support::sprint104_support::{read_fixture, write_manifest, write_support_json};

#[test]
fn final_verification_gate_passes_with_explicit_warnings_when_no_blocking_findings_remain() {
    let bundle = Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&DualAgentWorkflowConfig::default())
        .expect("run");
    let actual = to_value(&bundle.final_verification_gate).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint104_data/final_verification_gate_expected.json");
    assert_eq!(actual, expected);
    assert!(bundle.final_verification_gate.final_verification_passed);
    assert_eq!(
        bundle.final_verification_gate.gate_status,
        "FinalVerificationPassedWithWarnings"
    );
}

#[test]
fn final_verification_gate_blocks_on_safety_violation() {
    let mut config = DualAgentWorkflowConfig::default();
    config.preserve_safety_guards = false;
    let bundle = Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        bundle.final_verification_gate.gate_status,
        "FinalVerificationBlocked"
    );
}

#[test]
fn final_verification_gate_blocks_on_architecture_regression() {
    let manifest = write_manifest(
        "final-verification-gate-architecture",
        "changed_files.json",
        &["src/central_ai_core.rs"],
    );
    let mut config = DualAgentWorkflowConfig::default();
    config.changed_file_manifest_paths = Some(vec![manifest]);
    let bundle = Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        bundle.final_verification_gate.gate_status,
        "FinalVerificationBlocked"
    );
}

#[test]
fn final_verification_gate_blocks_when_workspace_attempt_status_is_missing() {
    let workspace_truth_path = write_support_json(
        "final-verification-gate-workspace-truth",
        "workspace_truth.json",
        &serde_json::json!({
            "truth_status": "WorkspaceTruthImported",
            "can_claim_full_acceptance": false
        }),
    );
    let mut config = DualAgentWorkflowConfig::default();
    config.workspace_truth_paths = Some(vec![workspace_truth_path]);
    let bundle = Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&config)
        .expect("run");
    assert!(
        !bundle
            .test_coverage_verification_report
            .workspace_attempt_reported
    );
    assert_eq!(
        bundle.test_coverage_verification_report.test_status,
        "TestCoverageInsufficient"
    );
    assert_eq!(
        bundle.final_verification_gate.gate_status,
        "FinalVerificationBlocked"
    );
}
