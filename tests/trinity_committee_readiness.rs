mod common;
#[path = "support/sprint59_support.rs"]
mod sprint59_support;

use soma_zero::{SystemIntegrationReviewRunner, TrinityCommitteeReadinessStatus};

#[test]
fn trinity_readiness_requires_exactly_three_active_personas() {
    let config = sprint59_support::review_config_from_example(
        "soma_system_review_full.toml",
        "trinity-readiness-full",
    );
    let bundle = SystemIntegrationReviewRunner::default()
        .run(&config)
        .expect("run trinity review");
    assert_eq!(
        bundle.trinity_readiness_report.readiness_status,
        TrinityCommitteeReadinessStatus::Ready
    );
    assert!(bundle.trinity_readiness_report.all_three_active);
    assert!(bundle.trinity_readiness_report.no_extra_active_personas);
}

#[test]
fn missing_trinity_loop_blocks_committee_readiness() {
    let config = sprint59_support::review_config_from_example(
        "soma_system_review_committee_gap.toml",
        "trinity-readiness-gap",
    );
    let bundle = SystemIntegrationReviewRunner::default()
        .run(&config)
        .expect("run committee-gap review");
    assert_eq!(
        bundle.trinity_readiness_report.readiness_status,
        TrinityCommitteeReadinessStatus::Blocked
    );
}
