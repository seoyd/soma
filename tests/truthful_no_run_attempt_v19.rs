mod support;

use soma_zero::{CommandObservation, TruthfulNoRunAttemptV19, build_truthful_no_run_attempt_v19};
use support::sprint118_support::{read_fixture, run_sprint118};

#[test]
fn truthful_no_run_attempt_handles_not_run_timeout_and_pass() {
    let bundle = run_sprint118(
        "soma_truthful_no_run_attempt_v19.toml",
        "truthful-no-run-attempt-v19",
    );
    let expected: TruthfulNoRunAttemptV19 =
        read_fixture("sprint118_data/truthful_no_run_attempt_expected.json");
    assert_eq!(bundle.truthful_no_run_attempt_v19, expected);
    let timeout = build_truthful_no_run_attempt_v19(
        Some(&CommandObservation {
            attempted: true,
            finished: false,
            passed: None,
            duration_ms: Some(1),
            timeout_ms: Some(420000),
            exit_code: Some(124),
            timed_out: true,
            stdout: String::new(),
        }),
        Some(420000),
    );
    assert!(!timeout.recovered);
    let success = build_truthful_no_run_attempt_v19(
        Some(&CommandObservation {
            attempted: true,
            finished: true,
            passed: Some(true),
            duration_ms: Some(1),
            timeout_ms: Some(420000),
            exit_code: Some(0),
            timed_out: false,
            stdout: String::new(),
        }),
        Some(420000),
    );
    assert!(success.recovered);
}
