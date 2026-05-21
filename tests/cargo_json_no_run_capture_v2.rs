mod support;

use support::sprint106_support::run_sprint106;

#[test]
fn cargo_json_capture_counts_messages_and_targets() {
    let bundle = run_sprint106(
        "soma_cargo_json_no_run_capture_v2.toml",
        "cargo_json_no_run_capture_v2",
    );
    let report = bundle.cargo_json_no_run_capture_v2;
    assert!(report.message_count > 0);
    assert!(report.compiler_artifact_count > 0);
    assert!(report.test_executable_count > 0);
    assert!(!report.last_artifacts.is_empty());
    assert!(!report.last_targets.is_empty());
}
