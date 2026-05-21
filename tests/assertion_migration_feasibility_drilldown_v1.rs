mod support;

use std::collections::BTreeMap;

use serde_json::Value;
use soma_zero::{
    AssertionDestinationCandidateReportV1, AssertionDestinationCandidateRowV1,
    TargetAssertionInventoryReportV1, build_assertion_migration_feasibility_drilldown_report_v1,
    build_equivalent_coverage_feasibility_drilldown_report_v1,
};
use support::sprint114_support::{read_fixture, run_sprint114};

#[test]
fn assertion_migration_drilldown_supports_blocked_and_feasible_cases() {
    let bundle = run_sprint114(
        "soma_assertion_migration_feasibility_drilldown_v1.toml",
        "assertion-migration-feasibility-drilldown-v1",
    );
    let expected: Value =
        read_fixture("sprint114_data/assertion_migration_feasibility_expected.json");
    let report = bundle.assertion_migration_feasibility_drilldown_report_v1;
    assert_eq!(
        report.feasibility_status,
        expected["feasibility_status"].as_str().unwrap()
    );
    assert!(!report.feasible);
    assert!(!report.blockers.is_empty());

    let inventory = TargetAssertionInventoryReportV1 {
        report_id: "inventory".to_string(),
        candidate_targets: vec![
            "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
        ],
        assertion_count_by_target: BTreeMap::from([(
            "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
            2,
        )]),
        assertion_kinds_by_target: BTreeMap::new(),
        assertion_dependencies: BTreeMap::new(),
        migration_complexity: BTreeMap::from([(
            "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
            "Low".to_string(),
        )]),
        inventory_status: "AssertionInventoryReady".to_string(),
        reason_codes: vec![],
    };
    let destinations = AssertionDestinationCandidateReportV1 {
        report_id: "dest".to_string(),
        destination_candidates: vec![AssertionDestinationCandidateRowV1 {
            candidate_target: "tests/shared_fixture_harness_application_v1.rs".to_string(),
            existing_coverage: "enough".to_string(),
            risk: "Low".to_string(),
            capacity: 2,
            destination_target_required: true,
        }],
        status: "ready".to_string(),
        reason_codes: vec![],
    };
    let feasible =
        build_assertion_migration_feasibility_drilldown_report_v1(&inventory, &destinations);
    assert!(feasible.feasible);
    assert_eq!(feasible.feasibility_status, "AssertionMigrationFeasible");

    let eq = build_equivalent_coverage_feasibility_drilldown_report_v1(
        &destinations,
        true,
        Some(vec!["gap".to_string()]),
    );
    assert!(!eq.feasible);
    assert_eq!(eq.status, "EquivalentCoverageBlocked");
}
