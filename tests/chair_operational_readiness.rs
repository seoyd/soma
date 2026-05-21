mod common;
#[path = "support/sprint59_support.rs"]
mod sprint59_support;

use soma_zero::{ChairOperationalReadinessStatus, SystemIntegrationReviewRunner};

#[test]
fn chair_readiness_is_ready_for_full_fixture() {
    let config = sprint59_support::review_config_from_example(
        "soma_system_review_full.toml",
        "chair-readiness-full",
    );
    let bundle = SystemIntegrationReviewRunner::default()
        .run(&config)
        .expect("run full chair review");
    assert_eq!(
        bundle.chair_readiness_report.readiness_status,
        ChairOperationalReadinessStatus::Ready
    );
    assert!(bundle.chair_readiness_report.risk_handoff_available);
    assert!(bundle.chair_readiness_report.no_bypass_detected);
}

#[test]
fn missing_chair_artifact_blocks_readiness() {
    let config = sprint59_support::review_config_from_example(
        "soma_system_review_chair_gap.toml",
        "chair-readiness-gap",
    );
    let bundle = SystemIntegrationReviewRunner::default()
        .run(&config)
        .expect("run chair-gap review");
    assert_eq!(
        bundle.chair_readiness_report.readiness_status,
        ChairOperationalReadinessStatus::Blocked
    );
}
