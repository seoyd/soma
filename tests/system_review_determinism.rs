mod common;
#[path = "support/sprint59_support.rs"]
mod sprint59_support;

use soma_zero::SystemIntegrationReviewRunner;

#[test]
fn system_review_runner_is_deterministic_for_same_fixture() {
    let config = sprint59_support::review_config_from_example(
        "soma_system_review_full.toml",
        "system-review-determinism",
    );
    let first = SystemIntegrationReviewRunner::default()
        .run(&config)
        .expect("first review run");
    let second = SystemIntegrationReviewRunner::default()
        .run(&config)
        .expect("second review run");
    assert_eq!(first, second);
}
