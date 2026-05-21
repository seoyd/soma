mod support;

use support::sprint106_support::{read_fixture, run_sprint106};

#[test]
fn fixture_setup_cost_attribution_covers_shared_harness_diagnostics() {
    let bundle = run_sprint106(
        "soma_fixture_setup_cost_attribution_v2.toml",
        "fixture_setup_cost_attribution_v2",
    );
    let report = bundle.fixture_setup_cost_attribution_v2;
    assert!(!report.duplicate_json_loaders.is_empty());
    assert!(!report.duplicate_toml_loaders.is_empty());
    assert!(!report.duplicate_output_dir_setup.is_empty());
    assert!(!report.shared_harness_opportunities.is_empty());

    let actual =
        serde_json::to_value(&bundle.shared_fixture_harness_expansion_plan_v2).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint106_data/shared_fixture_harness_plan_expected.json");
    assert_eq!(actual, expected);
    assert!(
        bundle
            .shared_fixture_harness_expansion_plan_v2
            .determinism_preserved
    );
}
