mod support;

use support::sprint110_support::run_sprint110;

#[test]
fn shared_helper_expansion_matches_expected_guards() {
    let bundle = run_sprint110(
        "soma_shared_fixture_harness_expansion_v4.toml",
        "shared-fixture-harness-expansion-v4",
    );
    assert_eq!(
        bundle
            .shared_fixture_harness_expansion_application_report_v4
            .application_status,
        "SharedFixtureHarnessExpanded"
    );
    assert_eq!(
        bundle
            .shared_render_helper_expansion_report_v4
            .application_status,
        "SharedRenderHelperExpanded"
    );
    assert_eq!(
        bundle
            .shared_output_dir_helper_expansion_report_v4
            .application_status,
        "SharedOutputDirHelperExpanded"
    );
    assert_eq!(
        bundle
            .shared_toml_builder_expansion_report_v4
            .application_status,
        "SharedTomlBuilderExpanded"
    );
    assert!(
        bundle
            .shared_toml_builder_expansion_report_v4
            .remote_path_rejection_preserved
    );
}
