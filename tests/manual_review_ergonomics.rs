mod common;
#[path = "support/sprint60_support.rs"]
mod sprint60_support;

use serde_json::json;
use soma_zero::{EvidenceHardeningRunner, ManualReviewErgonomicsStatus};

#[test]
fn manual_review_counts_and_status_are_computed() {
    let config = sprint60_support::config_from_example(
        "soma_review_ergonomics.toml",
        "manual-review-ergonomics",
    );
    let report = EvidenceHardeningRunner::default()
        .run(&config)
        .expect("run review ergonomics")
        .manual_review_ergonomics_report;
    assert_eq!(report.pending_review_count, 2);
    assert_eq!(report.risk_blocked_count, 1);
    assert_eq!(
        report.ergonomics_status,
        ManualReviewErgonomicsStatus::NeedsBetterOwnerDiscipline
    );
}

#[test]
fn missing_reasons_are_detected() {
    let mut config = sprint60_support::config_from_example(
        "soma_review_ergonomics.toml",
        "manual-review-missing-reasons",
    );
    config.owner_review_queue_paths = vec![sprint60_support::write_support_json(
        "manual-review-missing-reasons-support",
        "queue.json",
        &json!({
            "queue_id": "owner-review-unclear",
            "pending_items": [{
                "review_id": "r1",
                "candidate_id": "cand-1",
                "current_status": "PendingReview",
                "allowed_owner_actions": [],
                "forbidden_owner_actions": []
            }],
            "reviewed_items": [],
            "deferred_items": [],
            "dismissed_items": [],
            "paper_confirmed_items": [],
            "blocked_items": [],
            "expired_items": []
        }),
    )];
    let report = EvidenceHardeningRunner::default()
        .run(&config)
        .expect("run review ergonomics missing reasons")
        .manual_review_ergonomics_report;
    assert!(report.missing_reason_count > 0);
    assert!(report.unclear_action_count > 0);
}
