mod support;

use soma_zero::{
    CommandObservation, RealFullWorkspaceObservationAttemptV17,
    build_real_full_workspace_observation_attempt_v17,
};
use support::sprint116_support::{read_fixture, run_sprint116};

#[test]
fn real_full_workspace_observation_attempt_v17_not_run_matches_expected() {
    let bundle = run_sprint116(
        "soma_real_full_workspace_observation_attempt_v17.toml",
        "real-full-workspace-observation-attempt-v17",
    );
    let expected: RealFullWorkspaceObservationAttemptV17 =
        read_fixture("sprint116_data/real_full_observation_expected.json");
    assert_eq!(bundle.real_full_workspace_observation_attempt_v17, expected);
}

#[test]
fn real_full_timeout_cannot_accept_and_finished_pass_can_accept() {
    let timed_out = build_real_full_workspace_observation_attempt_v17(
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
    assert!(timed_out.supporting_only);
    assert!(!timed_out.full_accepted);

    let passed = build_real_full_workspace_observation_attempt_v17(
        Some(&CommandObservation {
            attempted: true,
            finished: true,
            passed: Some(true),
            duration_ms: Some(10_000),
            timeout_ms: Some(420_000),
            exit_code: Some(0),
            timed_out: false,
            stdout: String::new(),
        }),
        Some(420_000),
    );
    assert!(!passed.supporting_only);
    assert!(passed.full_accepted);
}
