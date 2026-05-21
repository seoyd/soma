mod support;

use support::sprint107_support::{read_fixture, run_sprint107};

#[test]
fn shared_fixture_harness_application_is_deterministic() {
    let bundle = run_sprint107(
        "soma_shared_fixture_harness_application_v1.toml",
        "shared-fixture-harness-application-v1",
    );
    let actual =
        serde_json::to_value(&bundle.shared_fixture_harness_application_report_v1).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint107_data/shared_fixture_harness_application_expected.json");
    assert_eq!(actual, expected);
    assert!(
        bundle
            .shared_fixture_harness_application_report_v1
            .json_loader_applied
    );
    assert!(
        bundle
            .shared_fixture_harness_application_report_v1
            .deterministic_output_preserved
    );
}

#[test]
fn shared_output_dir_helper_preserves_cleanup_policy() {
    let bundle = run_sprint107(
        "soma_shared_output_dir_helper_application_v1.toml",
        "shared-output-dir-helper-application-v1",
    );
    let actual = serde_json::to_value(&bundle.shared_output_dir_helper_application_report_v1)
        .expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint107_data/shared_output_dir_helper_application_expected.json");
    assert_eq!(actual, expected);
    assert!(
        bundle
            .shared_output_dir_helper_application_report_v1
            .no_silent_deletion
    );
}

#[test]
fn shared_render_helper_preserves_stable_ordering() {
    let bundle = run_sprint107(
        "soma_shared_render_helper_application_v1.toml",
        "shared-render-helper-application-v1",
    );
    let actual =
        serde_json::to_value(&bundle.shared_render_helper_application_report_v1).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint107_data/shared_render_helper_application_expected.json");
    assert_eq!(actual, expected);
    assert!(
        bundle
            .shared_render_helper_application_report_v1
            .snapshot_order_preserved
    );
}

#[test]
fn shared_toml_builder_preserves_local_only_validation() {
    let bundle = run_sprint107(
        "soma_shared_toml_builder_application_v1.toml",
        "shared-toml-builder-application-v1",
    );
    let actual =
        serde_json::to_value(&bundle.shared_toml_builder_application_report_v1).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint107_data/shared_toml_builder_application_expected.json");
    assert_eq!(actual, expected);
    assert!(
        bundle
            .shared_toml_builder_application_report_v1
            .remote_path_rejection_preserved
    );
}
