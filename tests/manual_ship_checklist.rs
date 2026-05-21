mod common;
#[path = "support/sprint59_support.rs"]
mod sprint59_support;

use soma_zero::{ShipChecklistOverallStatus, SystemIntegrationReviewRunner};

#[test]
fn full_review_checklist_passes_with_warnings() {
    let config = sprint59_support::review_config_from_example(
        "soma_manual_ship_checklist.toml",
        "manual-ship-checklist-full",
    );
    let bundle = SystemIntegrationReviewRunner::default()
        .run(&config)
        .expect("run checklist review");
    assert!(bundle.manual_ship_acceptance_checklist.all_required_passed);
    assert_eq!(
        bundle.manual_ship_acceptance_checklist.overall_status,
        ShipChecklistOverallStatus::PassedWithWarnings
    );
}

#[test]
fn safety_block_review_fails_required_checklist_items() {
    let config = sprint59_support::review_config_from_example(
        "soma_system_review_safety_block.toml",
        "manual-ship-checklist-safety",
    );
    let bundle = SystemIntegrationReviewRunner::default()
        .run(&config)
        .expect("run checklist safety review");
    assert!(!bundle.manual_ship_acceptance_checklist.all_required_passed);
    assert!(bundle.manual_ship_acceptance_checklist.fail_count > 0);
}
