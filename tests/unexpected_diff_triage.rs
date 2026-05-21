#[path = "support/sprint69_support.rs"]
mod support;

use std::collections::BTreeMap;

use soma_zero::{
    ContractAlignmentAuditStatusV2, DiffRootCauseReportStatus, UnexpectedDiffClassification,
    UnexpectedDiffTriageStatus,
};

#[test]
fn unexpected_diff_triage_classifies_two_cases_and_explains_contract_alignment() {
    let bundle = support::run_triage("soma_unexpected_diff_triage.toml", "unexpected-diff-triage");

    assert_eq!(
        bundle.unexpected_diff_triage_report.triage_status,
        UnexpectedDiffTriageStatus::UnexpectedDiffExplained
    );
    assert_eq!(
        bundle.unexpected_diff_triage_report.unexpected_diff_count,
        2
    );
    assert_eq!(
        bundle.snapshot_diff_classification_report.classified_count,
        2
    );
    assert_eq!(bundle.snapshot_diff_classification_report.unknown_count, 0);

    let by_key = bundle
        .snapshot_diff_classification_report
        .items
        .iter()
        .map(|item| {
            (
                format!("{}:{}", item.model_id, item.model_version),
                item.classification,
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        by_key.get("ext-model-a:1.1.0"),
        Some(&UnexpectedDiffClassification::LeaderboardStatusChange)
    );
    assert_eq!(
        by_key.get("ext-model-b:1.0.0"),
        Some(&UnexpectedDiffClassification::RiskMetricChange)
    );

    assert_eq!(
        bundle.contract_alignment_audit_v2.audit_status,
        ContractAlignmentAuditStatusV2::ContractAligned
    );
    assert_eq!(bundle.contract_alignment_audit_v2.aligned_count, 2);
    assert_eq!(bundle.contract_alignment_audit_v2.changed_count, 0);

    let contract_item = bundle
        .contract_alignment_audit_v2
        .items
        .iter()
        .find(|item| item.model_id == "ext-model-a" && item.model_version == "1.1.0")
        .expect("contract item");
    assert!(contract_item.used_summary_card_fallback);

    assert_eq!(
        bundle.diff_root_cause_report.report_status,
        DiffRootCauseReportStatus::RootCausesReady
    );
    let root_causes = bundle
        .diff_root_cause_report
        .items
        .iter()
        .find(|item| item.model_id == "ext-model-b" && item.model_version == "1.0.0")
        .expect("root cause item");
    assert!(
        root_causes
            .root_causes
            .iter()
            .any(|item| item.contains("risk_status=Low -> Critical"))
    );
}
