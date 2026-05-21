mod support;

use soma_zero::CargoJsonProgressCaptureV5;
use support::sprint111_support::{read_fixture, run_sprint111};

#[test]
fn cargo_json_progress_capture_v5_matches_fixture_and_stays_diagnostic() {
    let bundle = run_sprint111(
        "soma_cargo_json_progress_capture_v5.toml",
        "cargo-json-progress-capture-v5",
    );
    let expected: CargoJsonProgressCaptureV5 =
        read_fixture("sprint111_data/cargo_json_progress_v5_expected.json");
    assert_eq!(bundle.cargo_json_progress_capture_v5, expected);
    assert_eq!(
        bundle.cargo_json_progress_capture_v5.capture_status,
        "DiagnosticOnly"
    );
    assert!(bundle.cargo_json_progress_capture_v5.message_count > 0);
    assert!(
        bundle
            .cargo_json_progress_capture_v5
            .stalled_target_candidates
            .contains(&"tests/workspace_timeout_root_cause.rs".to_string())
    );
}
