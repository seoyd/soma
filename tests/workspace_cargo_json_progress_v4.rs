mod support;

use support::sprint110_support::run_sprint110;

#[test]
fn workspace_cargo_json_progress_v4_is_diagnostic_only_by_default() {
    let bundle = run_sprint110(
        "soma_workspace_cargo_json_progress_v4.toml",
        "workspace-cargo-json-progress-v4",
    );
    let report = bundle.workspace_cargo_json_progress_capture_v4;
    assert!(!report.attempted);
    assert_eq!(
        report.previous_capture_refs,
        vec!["workspace-cargo-json-progress-capture-v3".to_string()]
    );
    assert_eq!(report.capture_status, "CargoJsonProgressCaptureNotRun");
}
