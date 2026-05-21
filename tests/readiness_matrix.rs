mod common;
#[path = "support/sprint59_support.rs"]
mod sprint59_support;

use soma_zero::{
    ReadinessArea, ReadinessMatrixOverallStatus, ReadinessStatus, SystemIntegrationReviewRunner,
};

#[test]
fn safety_block_config_blocks_ui_and_secret_rows() {
    let config = sprint59_support::review_config_from_example(
        "soma_system_review_safety_block.toml",
        "system-review-safety-block",
    );
    let bundle = SystemIntegrationReviewRunner::default()
        .run(&config)
        .expect("run safety-block review");
    let ui_row = bundle
        .readiness_matrix
        .rows
        .iter()
        .find(|row| row.area == ReadinessArea::ControlTowerUI)
        .expect("ui row");
    let secret_row = bundle
        .readiness_matrix
        .rows
        .iter()
        .find(|row| row.area == ReadinessArea::SecretSafety)
        .expect("secret row");
    assert_eq!(ui_row.status, ReadinessStatus::Blocked);
    assert_eq!(secret_row.status, ReadinessStatus::Blocked);
    assert_eq!(
        bundle.readiness_matrix.overall_status,
        ReadinessMatrixOverallStatus::Blocked
    );
}
