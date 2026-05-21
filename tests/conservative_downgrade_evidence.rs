mod common;
#[path = "support/sprint69_support.rs"]
mod sprint69_support;

use soma_zero::{ConservativeDowngradeEvidenceAuditStatus, SnapshotDiffIntegrityStatus};

#[test]
fn downgrade_evidence_requires_regression_support_for_retirement() {
    let bundle =
        sprint69_support::run_coverage("soma_downgrade_evidence_audit.toml", "downgrade-audit");
    assert_eq!(
        bundle.conservative_downgrade_evidence_audit.audit_status,
        ConservativeDowngradeEvidenceAuditStatus::DowngradeEvidenceIncomplete
    );
    assert_eq!(
        bundle
            .conservative_downgrade_evidence_audit
            .enough_for_downgrade_count,
        0
    );
    assert_eq!(
        bundle
            .conservative_downgrade_evidence_audit
            .not_enough_count,
        1
    );

    let retire_item = bundle
        .conservative_downgrade_evidence_audit
        .items
        .iter()
        .find(|item| item.model_id == "ext-model-a" && item.model_version == "1.0.0")
        .expect("retire item");
    assert_eq!(
        retire_item.downgrade_recommendation.as_deref(),
        Some("RetireModelVersion")
    );
    assert!(
        retire_item
            .missing_evidence
            .iter()
            .any(|item| format!("{item:?}") == "RegressionEvidence")
    );
    assert!(!retire_item.enough_for_retirement);
}

#[test]
fn snapshot_diff_integrity_tracks_unexpected_diffs() {
    let bundle = sprint69_support::run_coverage(
        "soma_snapshot_diff_integrity.toml",
        "snapshot-diff-integrity",
    );
    assert_eq!(
        bundle.snapshot_diff_integrity_report.integrity_status,
        SnapshotDiffIntegrityStatus::UnexpectedDiff
    );
    assert_eq!(bundle.snapshot_diff_integrity_report.ready_count, 2);
    assert_eq!(
        bundle.snapshot_diff_integrity_report.unexpected_diff_count,
        2
    );

    let ext_model_b = bundle
        .snapshot_diff_integrity_report
        .items
        .iter()
        .find(|item| item.model_id == "ext-model-b" && item.model_version == "1.0.0")
        .expect("ext-model-b diff");
    assert_eq!(
        ext_model_b.diff_status,
        SnapshotDiffIntegrityStatus::UnexpectedDiff
    );
    assert!(ext_model_b.diff_available);
}
