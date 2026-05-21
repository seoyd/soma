mod support;

use soma_zero::{RemainingBlockerQueueV7Status, Sprint91KrxEvidenceRecoveryRunner};
use std::fs;
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn remaining_blocker_queue_v7_defaults_to_queue_reduced_for_warning_backed_reduction() {
    let config = sprint::sprint91_config_from_example(
        "soma_remaining_blocker_queue_v7.toml",
        "krx-remaining-queue-default",
    );
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_remaining_blocker_queue_v7(&config)
        .expect("report");
    assert_eq!(
        report.queue_status,
        RemainingBlockerQueueV7Status::QueueReduced
    );
    assert_eq!(report.primary_next_family, "KrxEvidence");
}

#[test]
fn remaining_blocker_queue_v7_can_be_ready_or_closed() {
    let mut ready = sprint::sprint91_config_from_example(
        "soma_remaining_blocker_queue_v7.toml",
        "krx-remaining-queue-ready",
    );
    ready.preserve_assertions = false;
    let ready_report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_remaining_blocker_queue_v7(&ready)
        .expect("ready report");
    assert_eq!(
        ready_report.queue_status,
        RemainingBlockerQueueV7Status::QueueReady
    );

    let mut closed = sprint::sprint91_config_from_example(
        "soma_remaining_blocker_queue_v7.toml",
        "krx-remaining-queue-closed",
    );
    let dir = harness::temp_output_dir_for_test("krx-remaining-queue-closed");
    let assertion_path = dir.join("krx_assertion_migration_expected.json");
    fs::write(
        &assertion_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "donor_files": ["tests/krx_collection_dry_run.rs"],
            "target_suite": "tests/krx_evidence_suite.rs",
            "high_risk_assertions_kept_separate": []
        }))
        .expect("json"),
    )
    .expect("write assertion sample");
    let queue_path = dir.join("queue_progress_v7_expected.json");
    fs::write(
        &queue_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "reduced_queue": [],
            "reduced_primary_next_family": "Done"
        }))
        .expect("json"),
    )
    .expect("write queue sample");
    closed
        .cargo_metadata_paths
        .retain(|value| !value.ends_with("krx_assertion_migration_expected.json"));
    closed
        .cargo_metadata_paths
        .push(assertion_path.display().to_string());
    closed
        .cargo_timings_paths
        .retain(|value| !value.ends_with("queue_progress_v7_expected.json"));
    closed
        .cargo_timings_paths
        .push(queue_path.display().to_string());
    let closed_report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_remaining_blocker_queue_v7(&closed)
        .expect("closed report");
    assert_eq!(
        closed_report.queue_status,
        RemainingBlockerQueueV7Status::QueueClosed
    );
}
