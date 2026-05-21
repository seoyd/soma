use soma_zero::{
    CommitteeReferenceDepthClosurePlanStatus, CommitteeScenarioRepresentativenessStatus,
    CommitteeSequenceDisagreementReportStatus, TrainingArtifactLineageCompletenessStatus,
    TrainingArtifactReferenceDepthStatus,
};

#[path = "support/sprint69_support.rs"]
mod support;

#[test]
fn sprint81_committee_depth_and_lineage_reports_are_specific() {
    let bundle =
        support::run_sprint81_bundle("soma_committee_reference_audit_v2.toml", "committee-depth");

    assert_eq!(
        bundle.committee_reference_depth_closure_plan.plan_status,
        CommitteeReferenceDepthClosurePlanStatus::ClosurePlanReady
    );
    assert_eq!(
        bundle
            .committee_scenario_representativeness_report
            .representativeness_status,
        CommitteeScenarioRepresentativenessStatus::RepresentativeEnoughForResearch
    );
    assert_eq!(
        bundle.committee_sequence_disagreement_report.report_status,
        CommitteeSequenceDisagreementReportStatus::DisagreementReportReady
    );
    assert_eq!(
        bundle
            .training_artifact_lineage_completeness_report
            .completeness_status,
        TrainingArtifactLineageCompletenessStatus::LineageComplete
    );
    assert_eq!(
        bundle.training_artifact_reference_depth_report.depth_status,
        TrainingArtifactReferenceDepthStatus::ReferenceDepthReady
    );
}
