#[path = "support/sprint69_support.rs"]
mod support;

use std::collections::BTreeMap;

use soma_zero::{
    DowngradeEvidenceClosureAction, DowngradeEvidenceClosurePlanStatus,
    ModelVersionReviewDispositionKind, OwnerReviewClosureActionV2, OwnerReviewClosureV2Status,
    TraceWarningReductionStatus,
};

#[test]
fn owner_review_closure_reduces_warnings_but_keeps_evidence_conservative() {
    let bundle = support::run_triage("soma_unexpected_diff_triage.toml", "owner-review-closure");

    assert_eq!(
        bundle.owner_review_closure_v2_report.closure_status,
        OwnerReviewClosureV2Status::OwnerReviewClosureReady
    );
    assert_eq!(
        bundle
            .owner_review_closure_v2_report
            .missing_owner_review_count,
        3
    );
    assert_eq!(
        bundle
            .owner_review_closure_v2_report
            .resolved_missing_owner_review_count,
        3
    );
    assert_eq!(
        bundle
            .owner_review_closure_v2_report
            .pending_owner_review_count,
        0
    );

    let closure_by_key = bundle
        .owner_review_closure_v2_report
        .items
        .iter()
        .map(|item| {
            (
                format!("{}:{}", item.model_id, item.model_version),
                (item.closure_action, item.safe_to_close),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        closure_by_key.get("ext-model-a:1.1.0"),
        Some(&(OwnerReviewClosureActionV2::KeepResearchCandidate, true))
    );
    assert_eq!(
        closure_by_key.get("ext-model-b:1.0.0"),
        Some(&(OwnerReviewClosureActionV2::MarkDiagnosticOnly, true))
    );
    assert_eq!(
        closure_by_key.get("ext-model-a:1.0.0"),
        Some(&(OwnerReviewClosureActionV2::NeedMoreEvidence, false))
    );

    assert_eq!(
        bundle
            .trace_completeness_warning_reduction_report
            .report_status,
        TraceWarningReductionStatus::TraceWarningsReduced
    );
    assert_eq!(
        bundle
            .trace_completeness_warning_reduction_report
            .original_warning_count,
        4
    );
    assert_eq!(
        bundle
            .trace_completeness_warning_reduction_report
            .reduced_warning_count,
        3
    );
    assert_eq!(
        bundle
            .trace_completeness_warning_reduction_report
            .remaining_warning_count,
        1
    );

    assert_eq!(
        bundle.downgrade_evidence_closure_plan.plan_status,
        DowngradeEvidenceClosurePlanStatus::DowngradeEvidenceStillIncomplete
    );
    let downgrade_item = bundle
        .downgrade_evidence_closure_plan
        .items
        .iter()
        .find(|item| item.model_id == "ext-model-a" && item.model_version == "1.0.0")
        .expect("downgrade item");
    assert_eq!(
        downgrade_item.closure_action,
        DowngradeEvidenceClosureAction::NeedMoreEvidence
    );

    let dispositions = bundle
        .model_version_review_disposition_report
        .items
        .iter()
        .map(|item| {
            (
                format!("{}:{}", item.model_id, item.model_version),
                item.disposition,
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        dispositions.get("ext-model-a:1.0.0"),
        Some(&ModelVersionReviewDispositionKind::NeedMoreEvidence)
    );
    assert_eq!(
        dispositions.get("ext-model-a:1.1.0"),
        Some(&ModelVersionReviewDispositionKind::KeepResearchCandidate)
    );
    assert_eq!(
        dispositions.get("ext-model-a:1.2.0"),
        Some(&ModelVersionReviewDispositionKind::KeepResearchCandidate)
    );
    assert_eq!(
        dispositions.get("ext-model-b:1.0.0"),
        Some(&ModelVersionReviewDispositionKind::MarkDiagnosticOnly)
    );
}
