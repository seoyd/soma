#[path = "support/sprint69_support.rs"]
mod support;

use serde_json::Value;
use soma_zero::{ControlTowerWarningKind, ControlTowerWarningReductionStatus};

#[test]
fn warnings_are_reduced_without_hiding_remaining_risks() {
    let report = support::run_sprint74_bundle(
        "soma_control_tower_warning_reduce.toml",
        "control-tower-warning-reduction",
    )
    .control_tower_warning_reduction_report;
    let expected: Value = support::read_json(support::example_path(
        "sprint74_data/expected_warning_reduction.json",
    ));
    assert_eq!(
        report.warnings_before,
        expected["warnings_before"].as_u64().unwrap() as usize
    );
    assert_eq!(
        report.warnings_after,
        expected["warnings_after"].as_u64().unwrap() as usize
    );
    assert_eq!(
        report.reduced_count,
        expected["reduced_count"].as_u64().unwrap() as usize
    );
    assert_eq!(
        report.remaining_count,
        expected["remaining_count"].as_u64().unwrap() as usize
    );
    assert_eq!(
        report.reduction_status,
        ControlTowerWarningReductionStatus::WarningsReduced
    );
    assert!(report.items.iter().any(|item| item.warning_kind
        == ControlTowerWarningKind::DirectWatchMonitoringOnly
        && item.after_present));
}
