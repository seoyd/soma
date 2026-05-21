mod common;
#[path = "support/sprint60_support.rs"]
mod sprint60_support;

use soma_zero::{
    EvidenceGapPrimaryGap, EvidenceHardeningRecommendation, EvidenceHardeningRunner,
    Mamba3ApplicationTimingDecision, ManualReviewErgonomicsStatus, UIFrameworkDecisionStatus,
};

#[test]
fn evidence_hardening_config_rejects_remote_paths() {
    let mut config = sprint60_support::config_from_example(
        "soma_evidence_hardening.toml",
        "evidence-hardening-remote",
    );
    config.kis_smoke_report_paths = vec!["https://example.com/report.json".to_string()];
    let err = config
        .validate()
        .expect_err("remote path should be rejected");
    assert!(err.contains("local"));
}

#[test]
fn full_evidence_hardening_builds_bundle() {
    let config =
        sprint60_support::config_from_example("soma_evidence_hardening.toml", "evidence-hardening");
    let bundle = EvidenceHardeningRunner::default()
        .run(&config)
        .expect("run evidence hardening");
    assert_eq!(
        bundle.evidence_depth_gap_report.primary_gap,
        EvidenceGapPrimaryGap::NeedMoreKISEvidence
    );
    assert_eq!(
        bundle.manual_review_ergonomics_report.ergonomics_status,
        ManualReviewErgonomicsStatus::NeedsBetterOwnerDiscipline
    );
    assert_eq!(
        bundle.ui_framework_decision_report.decision_status,
        UIFrameworkDecisionStatus::KeepStaticDashboardNow
    );
    assert_eq!(
        bundle.mamba3_application_timing_report.final_decision,
        Mamba3ApplicationTimingDecision::BuildSequenceDatasetFirst
    );
    assert!(
        bundle
            .final_recommendations
            .contains(&EvidenceHardeningRecommendation::NeedMoreEvidence)
    );
}
