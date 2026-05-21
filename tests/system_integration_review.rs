mod common;
#[path = "support/sprint59_support.rs"]
mod sprint59_support;

use std::path::PathBuf;

use soma_zero::{
    ControlTowerUiReadinessStatus, EndToEndPaperLoopAcceptanceStatus, ReadinessMatrixOverallStatus,
    SystemIntegrationReviewRunner, SystemShipGateRecommendation, SystemShipGateStatus,
};

#[test]
fn full_system_review_builds_bundle() {
    let config = sprint59_support::review_config_from_example(
        "soma_system_review_full.toml",
        "system-review-full",
    );
    let bundle = SystemIntegrationReviewRunner::default()
        .run(&config)
        .expect("run full system review");
    assert_eq!(
        bundle.readiness_matrix.overall_status,
        ReadinessMatrixOverallStatus::ReadyWithWarnings
    );
    assert_eq!(
        bundle.control_tower_ui_readiness_report.readiness_status,
        ControlTowerUiReadinessStatus::Ready
    );
    assert_eq!(
        bundle
            .end_to_end_paper_loop_acceptance_report
            .acceptance_status,
        EndToEndPaperLoopAcceptanceStatus::Passed
    );
    assert_eq!(
        bundle.system_ship_gate_report.final_status,
        SystemShipGateStatus::ReadyWithManualWarnings
    );
    assert_eq!(
        bundle.system_ship_gate_report.final_recommendation,
        SystemShipGateRecommendation::ReviewWarningsManually
    );
    assert!(
        PathBuf::from(&config.output_root)
            .join(&config.review_id)
            .join("summary.txt")
            .exists()
    );
}
