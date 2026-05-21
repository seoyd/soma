mod support;

use serde_json::Value;
use support::sprint114_support::{read_fixture, run_sprint114};

#[test]
fn suspect_target_decomposition_maps_targets_to_pressures() {
    let bundle = run_sprint114(
        "soma_suspect_target_decomposition_v1.toml",
        "suspect-target-decomposition-v1",
    );
    let expected: Value = read_fixture("sprint114_data/suspect_target_decomposition_expected.json");
    let report = bundle.suspect_target_decomposition_report_v1;
    assert_eq!(
        report.decomposition_status,
        expected["decomposition_status"].as_str().unwrap()
    );
    assert!(
        report.per_target_pressure["tests/control_tower_workspace_timeout_root_cause_panel.rs"]
            .contains(&"CliSmoke".to_string())
    );
    assert!(
        report.per_target_pressure["tests/workspace_timeout_root_cause.rs"]
            .contains(&"MacroExpansion".to_string())
    );
    assert!(
        report.per_target_pressure["tests/shared_fixture_harness_application_v1.rs"]
            .contains(&"FixtureSetup".to_string())
    );
}
