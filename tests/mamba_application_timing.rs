mod common;
#[path = "support/sprint60_support.rs"]
mod sprint60_support;

use serde_json::json;
use soma_zero::{EvidenceHardeningRunner, Mamba3ApplicationStage, Mamba3ApplicationTimingDecision};

#[test]
fn sequence_gate_blocks_mamba_timing() {
    let config = sprint60_support::config_from_example(
        "soma_mamba_application_timing.toml",
        "mamba-application-timing",
    );
    let report = EvidenceHardeningRunner::default()
        .run(&config)
        .expect("run mamba timing")
        .mamba3_application_timing_report;
    assert_eq!(report.current_stage, Mamba3ApplicationStage::Deferred);
    assert_eq!(
        report.final_decision,
        Mamba3ApplicationTimingDecision::BuildSequenceDatasetFirst
    );
    assert!(report.runtime_deferred);
}

#[test]
fn external_prototype_is_only_allowed_if_sequence_and_evidence_gates_pass() {
    let mut config = sprint60_support::config_from_example(
        "soma_mamba_application_timing.toml",
        "mamba-application-timing-ready",
    );
    config.sequence_readiness_report_paths = vec![
        sprint60_support::absolutize("examples/sprint55_data/sequence_readiness_ready.json")
            .display()
            .to_string(),
    ];
    config.min_official_rows = 12;
    config.min_complete_rows = 2;
    config.min_outcome_links = 2;
    config.owner_review_queue_paths = vec![sprint60_support::write_support_json(
        "mamba-application-timing-ready-support",
        "owner_queue.json",
        &json!({
            "queue_id": "owner-review-clean",
            "pending_items": [],
            "reviewed_items": [],
            "deferred_items": [],
            "dismissed_items": [],
            "paper_confirmed_items": [],
            "blocked_items": [],
            "expired_items": [],
            "reason_codes": ["OwnerReviewQueueBuilt"]
        }),
    )];
    let report = EvidenceHardeningRunner::default()
        .run(&config)
        .expect("run mamba timing ready")
        .mamba3_application_timing_report;
    assert_eq!(
        report.current_stage,
        Mamba3ApplicationStage::ExternalPrototypeOnlyIfReady
    );
}
