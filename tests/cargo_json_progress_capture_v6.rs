mod support;

use soma_zero::CargoJsonProgressCaptureV6;
use support::sprint112_support::{read_fixture, run_sprint112};

#[test]
fn cargo_json_progress_capture_v6_matches_fixture_and_stays_non_acceptance() {
    let bundle = run_sprint112(
        "soma_cargo_json_progress_capture_v6.toml",
        "cargo-json-progress",
    );
    let expected: CargoJsonProgressCaptureV6 =
        read_fixture("sprint112_data/cargo_json_progress_v6_expected.json");
    assert_eq!(bundle.cargo_json_progress_capture_v6, expected);
    assert_eq!(
        bundle.cargo_json_progress_capture_v6.status,
        "DiagnosticOnly"
    );
    assert!(bundle.cargo_json_progress_capture_v6.messages > 0);
    assert!(
        !bundle
            .workspace_diagnostic_evidence_matrix_v1
            .supports_acceptance
    );
}
