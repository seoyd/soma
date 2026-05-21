mod common;
#[path = "support/sprint61_support.rs"]
mod sprint61_support;

use soma_zero::{BoundedKISOfficialEvidenceClosureRunner, OwnerReviewDisciplineStatus};

#[test]
fn owner_review_discipline_flags_cleanup_needs() {
    let config = sprint61_support::discipline_config_from_example(
        "soma_owner_review_discipline_v2.toml",
        "owner-discipline",
    );
    let report = BoundedKISOfficialEvidenceClosureRunner::default()
        .run_owner_review_discipline_v2(&config)
        .expect("run owner discipline");
    assert_eq!(
        report.discipline_status,
        OwnerReviewDisciplineStatus::NeedsReasons
    );
    assert!(report.missing_reason_inputs > 0);
}
