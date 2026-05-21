mod common;
#[path = "support/sprint61_support.rs"]
mod sprint61_support;

use soma_zero::{
    BoundedKISOfficialEvidenceClosureRunner, KISEvidenceClosureStatus, OwnerReviewDisciplineStatus,
    SequenceReadinessHardeningStatus,
};

#[test]
fn kis_evidence_plan_rejects_remote_paths() {
    let mut config = sprint61_support::plan_config_from_example(
        "soma_kis_evidence_expansion_plan_v2.toml",
        "plan-remote",
    );
    config.kis_evidence_depth_report_paths = vec!["https://example.com/report.json".to_string()];
    let err = config.validate().expect_err("remote path should fail");
    assert!(err.contains("local"));
}

#[test]
fn bounded_kis_evidence_closure_builds_bundle() {
    let config =
        sprint61_support::closure_config_from_example("soma_kis_evidence_closure.toml", "bundle");
    let bundle = BoundedKISOfficialEvidenceClosureRunner::default()
        .run_kis_evidence_closure(&config)
        .expect("run kis evidence closure");
    assert_eq!(
        bundle.kis_evidence_closure_report.closure_status,
        KISEvidenceClosureStatus::KISEvidenceExpanded
    );
    assert_eq!(
        bundle
            .sequence_dataset_readiness_hardening_report
            .readiness_status,
        SequenceReadinessHardeningStatus::ReadyForSequenceDatasetExport
    );
    assert_eq!(
        bundle.owner_review_discipline_v2_report.discipline_status,
        OwnerReviewDisciplineStatus::NeedsRiskExplanation
    );
    assert!(
        bundle
            .control_tower_refresh_summary
            .as_ref()
            .expect("refresh summary")
            .no_training_button
    );
}

#[test]
fn bounded_kis_evidence_closure_is_deterministic() {
    let config = sprint61_support::closure_config_from_example(
        "soma_kis_evidence_closure.toml",
        "bundle-deterministic",
    );
    let runner = BoundedKISOfficialEvidenceClosureRunner::default();
    let left = runner
        .run_kis_evidence_closure(&config)
        .expect("first bundle");
    let right = runner
        .run_kis_evidence_closure(&config)
        .expect("second bundle");
    assert_eq!(left, right);
}
