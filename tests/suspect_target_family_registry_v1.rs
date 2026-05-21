mod support;

use serde_json::Value;
use support::sprint113_support::{read_fixture, run_sprint113};

#[test]
fn suspect_target_family_registry_matches_expected_targets() {
    let bundle = run_sprint113(
        "soma_suspect_target_family_registry_v1.toml",
        "suspect-target-family-registry-v1",
    );
    let expected: Value =
        read_fixture("sprint113_data/suspect_target_family_registry_expected.json");
    let targets = &bundle.suspect_target_family_registry_v1.suspect_targets;
    assert_eq!(
        targets,
        &vec![
            "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
            "tests/shared_fixture_harness_application_v1.rs".to_string(),
            "tests/workspace_timeout_root_cause.rs".to_string(),
        ]
    );
    assert_eq!(
        bundle.suspect_target_family_registry_v1.registry_status,
        expected["registry_status"].as_str().unwrap()
    );
    assert!(
        bundle
            .suspect_target_family_registry_v1
            .already_retired_targets_excluded
    );
    assert!(
        bundle
            .suspect_target_family_registry_v1
            .sentinel_targets_excluded
    );
    assert!(!targets.iter().any(|target| target.contains("retired")));
    assert!(!targets.iter().any(|target| target.contains("sentinel")));
}
