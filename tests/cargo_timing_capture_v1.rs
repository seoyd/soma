mod support;

use support::sprint112_support::run_sprint112;

#[test]
fn cargo_timing_captures_keep_no_run_full_blocked_without_overclaim() {
    let bundle = run_sprint112("soma_cargo_check_timing_capture_v1.toml", "cargo-timing");
    assert_eq!(
        bundle.cargo_check_timing_capture_v1.status,
        "CargoTimingCapturePassed"
    );
    assert_eq!(
        bundle.cargo_build_timing_capture_v1.status,
        "CargoTimingCapturePassed"
    );
    assert_eq!(
        bundle.cargo_no_run_timing_capture_v1.status,
        "CargoTimingCaptureTimedOut"
    );
    assert_eq!(
        bundle.cargo_full_run_timing_capture_v1.status,
        "CargoTimingCaptureTimedOut"
    );
    assert!(!bundle.workspace_no_run_recovery_gate_v13.recovered);
    assert!(!bundle.workspace_full_acceptance_gate_v13.accepted);
}
