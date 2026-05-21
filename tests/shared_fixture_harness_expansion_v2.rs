mod support;

use support::sprint108_support::{read_fixture, run_sprint108};

#[test]
fn shared_helper_expansion_matches_expected_fixture_and_preserves_guards() {
    let bundle = run_sprint108(
        "soma_shared_fixture_harness_expansion_v2.toml",
        "shared-fixture-harness-expansion-v2",
    );
    let actual =
        serde_json::to_value(&bundle.shared_fixture_harness_expansion_application_report_v2)
            .expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint108_data/shared_fixture_harness_expansion_v2_expected.json");
    assert_eq!(actual, expected);
    assert_eq!(
        bundle
            .shared_render_helper_expansion_report_v2
            .application_status,
        "SharedRenderHelperExpanded"
    );
    assert_eq!(
        bundle
            .shared_output_dir_helper_expansion_report_v2
            .application_status,
        "SharedOutputDirHelperExpanded"
    );
    assert_eq!(
        bundle
            .shared_toml_builder_expansion_report_v2
            .application_status,
        "SharedTomlBuilderExpanded"
    );
    assert!(
        bundle
            .shared_toml_builder_expansion_report_v2
            .remote_path_rejection_preserved
    );
}
