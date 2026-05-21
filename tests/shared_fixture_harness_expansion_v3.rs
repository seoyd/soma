mod support;

use support::sprint109_support::{read_fixture, run_sprint109};

#[test]
fn shared_helper_expansion_matches_expected_fixture_and_preserves_guards() {
    let bundle = run_sprint109(
        "soma_shared_fixture_harness_expansion_v3.toml",
        "shared-fixture-harness-expansion-v3",
    );
    let actual =
        serde_json::to_value(&bundle.shared_fixture_harness_expansion_application_report_v3)
            .expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint109_data/shared_fixture_harness_expansion_v3_expected.json");
    assert_eq!(actual, expected);
    assert_eq!(
        bundle
            .shared_render_helper_expansion_report_v3
            .application_status,
        "SharedRenderHelperExpanded"
    );
    assert_eq!(
        bundle
            .shared_output_dir_helper_expansion_report_v3
            .application_status,
        "SharedOutputDirHelperExpanded"
    );
    assert_eq!(
        bundle
            .shared_toml_builder_expansion_report_v3
            .application_status,
        "SharedTomlBuilderExpanded"
    );
    assert!(
        bundle
            .shared_toml_builder_expansion_report_v3
            .remote_path_rejection_preserved
    );
}
