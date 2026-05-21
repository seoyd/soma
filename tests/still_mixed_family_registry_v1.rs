mod support;

use serde_json::Value;
use support::sprint114_support::{read_fixture, run_sprint114};

#[test]
fn still_mixed_family_registry_carries_forward_truth() {
    let bundle = run_sprint114(
        "soma_still_mixed_family_registry_v1.toml",
        "still-mixed-family-registry-v1",
    );
    let expected: Value = read_fixture("sprint114_data/still_mixed_family_registry_expected.json");
    let registry = bundle.still_mixed_family_registry_v1;
    assert_eq!(
        registry.registry_id,
        expected["registry_id"].as_str().unwrap()
    );
    assert!(
        registry
            .mixed_families
            .contains(&"IntegrationTestBinaryFanout".to_string())
    );
    assert!(
        registry
            .mixed_families
            .contains(&"LinkTimeCost".to_string())
    );
    assert!(
        registry
            .mixed_families
            .contains(&"MacroExpansionCost".to_string())
    );
    assert!(
        registry
            .already_isolated_families
            .contains(&"FixtureSetupFanout".to_string())
    );
    assert!(
        registry
            .suspect_targets
            .contains(&"tests/workspace_timeout_root_cause.rs".to_string())
    );
}
