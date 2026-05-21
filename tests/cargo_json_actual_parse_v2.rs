mod support;

use soma_zero::{
    CargoJsonActualParseReportV2, CommandObservation, build_cargo_json_actual_parse_report_v2,
};
use support::sprint117_support::{read_fixture, run_sprint117};

#[test]
fn cargo_json_actual_parse_captures_counts() {
    let bundle = run_sprint117(
        "soma_cargo_json_actual_parse_v2.toml",
        "cargo-json-actual-parse-v2",
    );
    let expected: CargoJsonActualParseReportV2 =
        read_fixture("sprint117_data/cargo_json_actual_parse_expected.json");
    assert_eq!(bundle.cargo_json_actual_parse_report_v2, expected);
    let report = build_cargo_json_actual_parse_report_v2(Some(&CommandObservation {
        attempted: true,
        finished: true,
        passed: Some(true),
        duration_ms: Some(1),
        timeout_ms: Some(1),
        exit_code: Some(0),
        timed_out: false,
        stdout: r#"{"reason":"compiler-artifact","target":{"name":"one"},"filenames":["a"]}
{"reason":"compiler-message","target":{"name":"one"}}
"#
        .to_string(),
    }));
    assert_eq!(report.compiler_artifact_count, 1);
    assert_eq!(report.compiler_message_count, 1);
}
