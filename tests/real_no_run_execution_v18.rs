mod support;

use soma_zero::{CommandObservation, build_real_no_run_execution_report_v18};
use support::sprint117_support::run_sprint117;

#[test]
fn real_no_run_execution_deferred_and_timeout_never_passes() {
    let bundle = run_sprint117(
        "soma_real_no_run_execution_v18.toml",
        "real-no-run-execution-v18",
    );
    assert_eq!(
        bundle.real_no_run_execution_report_v18.execution_status,
        "RealNoRunDeferred"
    );
    let timed_out = build_real_no_run_execution_report_v18(
        Some(&CommandObservation {
            attempted: true,
            finished: false,
            passed: None,
            duration_ms: Some(1000),
            timeout_ms: Some(420000),
            exit_code: Some(124),
            timed_out: true,
            stdout: String::new(),
        }),
        Some(420000),
        Some((0, 0)),
    );
    assert!(timed_out.timed_out);
    assert_ne!(timed_out.passed, Some(true));
}
