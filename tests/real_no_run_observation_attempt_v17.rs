mod support;

use soma_zero::{
    CommandObservation, RealNoRunObservationAttemptV17, build_real_no_run_observation_attempt_v17,
};
use support::sprint116_support::{read_fixture, run_sprint116};

#[test]
fn real_no_run_observation_attempt_v17_not_run_matches_expected() {
    let bundle = run_sprint116(
        "soma_real_no_run_observation_attempt_v17.toml",
        "real-no-run-observation-attempt-v17",
    );
    let expected: RealNoRunObservationAttemptV17 =
        read_fixture("sprint116_data/real_no_run_observation_expected.json");
    assert_eq!(bundle.real_no_run_observation_attempt_v17, expected);
}

#[test]
fn real_no_run_timeout_cannot_pass_and_completed_pass_works() {
    let timed_out = build_real_no_run_observation_attempt_v17(
        Some(&CommandObservation {
            attempted: true,
            finished: false,
            passed: None,
            duration_ms: Some(420_000),
            timeout_ms: Some(420_000),
            exit_code: Some(124),
            timed_out: true,
            stdout: String::new(),
        }),
        Some(420_000),
    );
    assert_eq!(timed_out.attempt_status, "NoRunTimedOut");
    assert_ne!(timed_out.passed, Some(true));

    let passed = build_real_no_run_observation_attempt_v17(
        Some(&CommandObservation {
            attempted: true,
            finished: true,
            passed: Some(true),
            duration_ms: Some(5_000),
            timeout_ms: Some(420_000),
            exit_code: Some(0),
            timed_out: false,
            stdout: String::new(),
        }),
        Some(420_000),
    );
    assert_eq!(passed.attempt_status, "NoRunCompleted");
    assert_eq!(passed.passed, Some(true));
}
