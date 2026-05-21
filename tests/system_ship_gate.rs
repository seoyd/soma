mod common;
#[path = "support/sprint59_support.rs"]
mod sprint59_support;

use soma_zero::{SystemIntegrationReviewRunner, SystemShipGateStatus};

#[test]
fn full_review_ship_gate_is_ready_with_manual_warnings() {
    let config = sprint59_support::review_config_from_example(
        "soma_system_ship_gate.toml",
        "system-ship-gate-full",
    );
    let bundle = SystemIntegrationReviewRunner::default()
        .run(&config)
        .expect("run ship gate review");
    assert_eq!(
        bundle.system_ship_gate_report.final_status,
        SystemShipGateStatus::ReadyWithManualWarnings
    );
}

#[test]
fn safety_block_review_blocks_ship_gate() {
    let config = sprint59_support::review_config_from_example(
        "soma_system_review_safety_block.toml",
        "system-ship-gate-safety",
    );
    let bundle = SystemIntegrationReviewRunner::default()
        .run(&config)
        .expect("run safety ship gate review");
    assert_eq!(
        bundle.system_ship_gate_report.final_status,
        SystemShipGateStatus::BlockedBySafety
    );
}
