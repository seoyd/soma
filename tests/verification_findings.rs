mod support;

use soma_zero::{
    DualAgentWorkflowConfig, Sprint104DualAgentPaperLifecycleRunner, VerificationFindingCategory,
    VerificationFindingSeverity, VerificationFindingStatus,
};
use support::sprint104_support::{
    run_default_sprint103_fixture, write_sprint103_bundle, write_support_json,
};

#[test]
fn verification_findings_include_fixed_and_known_warning_entries() {
    let bundle = Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&DualAgentWorkflowConfig::default())
        .expect("run");
    assert!(
        bundle
            .verification_findings
            .iter()
            .any(|finding| finding.finding_status == VerificationFindingStatus::Fixed)
    );
    assert!(bundle.verification_findings.iter().any(|finding| {
        finding.finding_status == VerificationFindingStatus::AcceptedAsKnownWarning
            && finding.category == VerificationFindingCategory::WorkspaceTruth
    }));
}

#[test]
fn verification_findings_report_blocking_safety_findings() {
    let mut config = DualAgentWorkflowConfig::default();
    config.preserve_safety_guards = false;
    let bundle = Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&config)
        .expect("run");
    assert!(bundle.verification_findings.iter().any(|finding| {
        finding.category == VerificationFindingCategory::SafetyInvariant
            && finding.severity == VerificationFindingSeverity::Blocking
            && finding.finding_status == VerificationFindingStatus::Open
    }));
}

#[test]
fn verification_findings_report_overclaim_as_major() {
    let workspace_truth_path = write_support_json(
        "verification-findings-overclaim",
        "workspace_truth.json",
        &serde_json::json!({
            "truth_status": "WorkspaceTruthImported",
            "full_workspace_finished": false,
            "full_workspace_passed": false,
            "can_claim_full_acceptance": true,
            "no_run_status": "reported",
            "full_workspace_status": "reported"
        }),
    );
    let mut config = DualAgentWorkflowConfig::default();
    config.workspace_truth_paths = Some(vec![workspace_truth_path]);
    let bundle = Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&config)
        .expect("run");
    assert!(bundle.verification_findings.iter().any(|finding| {
        finding.category == VerificationFindingCategory::Overclaim
            && finding.severity == VerificationFindingSeverity::Major
            && finding.finding_status == VerificationFindingStatus::Open
    }));
    assert_eq!(
        bundle
            .workspace_acceptance_truth_closure_plan_v5
            .closure_status,
        "WorkspaceTruthStillOpenV5"
    );
    assert!(
        !bundle
            .workspace_acceptance_truth_closure_plan_v5
            .can_claim_full_acceptance
    );
    assert!(
        !bundle
            .workspace_acceptance_attempt_v20
            .can_claim_full_acceptance
    );
}

#[test]
fn verification_findings_report_architecture_regression() {
    let manifest = support::sprint104_support::write_manifest(
        "verification-findings-architecture",
        "changed_files.json",
        &["src/central_ai_core.rs"],
    );
    let mut config = DualAgentWorkflowConfig::default();
    config.changed_file_manifest_paths = Some(vec![manifest]);
    let bundle = Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&config)
        .expect("run");
    assert!(bundle.verification_findings.iter().any(|finding| {
        finding.category == VerificationFindingCategory::ArchitectureRegression
            && finding.severity == VerificationFindingSeverity::Blocking
    }));
}

#[test]
fn verification_findings_can_reflect_bypass_attempts_in_loaded_sprint103_bundle() {
    let mut sprint103 = run_default_sprint103_fixture();
    sprint103
        .risk_governor_notrade_reason_audit
        .bypass_attempt_count = 1;
    let bundle_path = write_sprint103_bundle(
        "verification-findings-bypass",
        "sprint103_bundle.json",
        &sprint103,
    );
    let mut config = DualAgentWorkflowConfig::default();
    config.sprint103_bundle_paths = Some(vec![bundle_path]);
    let bundle = Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        bundle.risk_governor_batch_veto_report.bypass_attempt_count,
        1
    );
}
