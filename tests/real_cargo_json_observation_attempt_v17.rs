mod support;

use soma_zero::{
    CommandObservation, RealCargoJsonObservationAttemptV17,
    build_real_cargo_json_observation_attempt_v17,
};
use support::sprint116_support::{read_fixture, run_sprint116};

#[test]
fn real_cargo_json_observation_attempt_v17_not_run_matches_expected() {
    let bundle = run_sprint116(
        "soma_real_cargo_json_observation_attempt_v17.toml",
        "real-cargo-json-observation-attempt-v17",
    );
    let expected: RealCargoJsonObservationAttemptV17 =
        read_fixture("sprint116_data/cargo_json_observation_expected.json");
    assert_eq!(bundle.real_cargo_json_observation_attempt_v17, expected);
}

#[test]
fn cargo_json_parse_errors_are_counted() {
    let attempt = build_real_cargo_json_observation_attempt_v17(
        Some(&CommandObservation {
            attempted: true,
            finished: true,
            passed: Some(false),
            duration_ms: Some(1_000),
            timeout_ms: Some(420_000),
            exit_code: Some(101),
            timed_out: false,
            stdout: "{\"target\":{\"name\":\"one\"},\"filenames\":[\"a\"]}\nnot-json\n".to_string(),
        }),
        Some(420_000),
    );
    assert_eq!(attempt.parsed_message_count, 1);
    assert_eq!(attempt.parse_error_count, 1);
    assert_eq!(attempt.malformed_line_count, 1);
}
