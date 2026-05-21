#[path = "support/sprint69_support.rs"]
mod support;

use serde_json::json;
use soma_zero::{
    RealEvidenceFollowupRunner, RealEvidenceOperatorRunbookStatus, RealEvidenceRunbookStepKind,
};

#[test]
fn runbook_includes_copyable_real_evidence_steps() {
    let report =
        support::run_sprint74_bundle("soma_real_evidence_runbook.toml", "real-evidence-runbook")
            .real_evidence_operator_runbook;
    assert_eq!(
        report.runbook_status,
        RealEvidenceOperatorRunbookStatus::RunbookReady
    );
    assert!(
        report
            .steps
            .iter()
            .any(|step| step.step_kind == RealEvidenceRunbookStepKind::AttachLocalKISEvidence)
    );
    assert!(
        report
            .steps
            .iter()
            .any(|step| step.step_kind == RealEvidenceRunbookStepKind::RefreshControlTower)
    );
    assert!(report.blocked_steps.is_empty());
}

#[test]
fn missing_provenance_blocks_runbook() {
    let provenance_path = support::write_support_json(
        "real-evidence-runbook-blocked",
        "provenance.json",
        &json!({"records":[]}),
    );
    let mut config = support::sprint74_config_from_example(
        "soma_real_evidence_runbook.toml",
        "real-evidence-runbook-blocked-config",
    );
    config.kis_provenance_paths = vec![provenance_path];
    let report = RealEvidenceFollowupRunner::default()
        .run_runbook(&config)
        .expect("run runbook");
    assert_eq!(
        report.runbook_status,
        RealEvidenceOperatorRunbookStatus::BlockedByProvenance
    );
}
