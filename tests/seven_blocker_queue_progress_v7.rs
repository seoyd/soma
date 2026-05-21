mod support;

use soma_zero::{
    KrxEvidenceRealReductionStatus, SevenBlockerQueueProgressStatusV7,
    Sprint91KrxEvidenceRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn queue_progress_stays_unchanged_when_krx_evidence_is_not_genuinely_reduced() {
    let config = sprint::sprint91_config_from_example(
        "soma_seven_blocker_queue_progress_v7.toml",
        "krx-queue-progress-default",
    );
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_seven_blocker_queue_progress_v7(&config)
        .expect("report");
    assert_eq!(
        report.queue_status,
        SevenBlockerQueueProgressStatusV7::QueueUnchanged
    );
    assert_eq!(report.primary_next_family, "KrxEvidence");
}

#[test]
fn queue_progress_advances_to_dashboard_renderer_when_krx_evidence_is_reduced() {
    let mut config = sprint::sprint91_config_from_example(
        "soma_seven_blocker_queue_progress_v7.toml",
        "krx-queue-progress-advanced",
    );
    let assertion_path = sprint::write_support_json(
        "krx-queue-progress-advanced",
        "krx_assertion_migration_expected.json",
        &serde_json::json!({
            "donor_files": ["tests/krx_collection_dry_run.rs"],
            "target_suite": "tests/krx_evidence_suite.rs",
            "high_risk_assertions_kept_separate": []
        }),
    );
    config
        .cargo_metadata_paths
        .retain(|value| !value.ends_with("krx_assertion_migration_expected.json"));
    config.cargo_metadata_paths.push(assertion_path);
    let bundle = Sprint91KrxEvidenceRecoveryRunner::default()
        .run(&config)
        .expect("bundle");
    assert_eq!(
        bundle.krx_evidence_real_reduction_report.reduction_status,
        KrxEvidenceRealReductionStatus::KrxEvidenceRealReduced
    );
    assert_eq!(
        bundle.seven_blocker_queue_progress_report_v7.queue_status,
        SevenBlockerQueueProgressStatusV7::QueueAdvanced
    );
    assert_eq!(
        bundle
            .seven_blocker_queue_progress_report_v7
            .primary_next_family,
        "DashboardRenderer"
    );
}
