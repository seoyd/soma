mod support;

use soma_zero::{CommandObservation, build_real_cargo_json_execution_report_v18};
use support::sprint117_support::run_sprint117;

#[test]
fn real_cargo_json_execution_counts_parse_errors() {
    let bundle = run_sprint117(
        "soma_real_cargo_json_execution_v18.toml",
        "real-cargo-json-execution-v18",
    );
    assert_eq!(
        bundle.real_cargo_json_execution_report_v18.execution_status,
        "RealCargoJsonDeferred"
    );
    let report = build_real_cargo_json_execution_report_v18(
        Some(&CommandObservation {
            attempted: true,
            finished: true,
            passed: Some(false),
            duration_ms: Some(1_000),
            timeout_ms: Some(420_000),
            exit_code: Some(101),
            timed_out: false,
            stdout: r#"{"reason":"compiler-message","target":{"name":"one"}}
not-json
"#
            .to_string(),
        }),
        Some(420_000),
    );
    assert_eq!(report.parsed_json_message_count, 1);
    assert_eq!(report.parse_error_count, 1);
    assert_eq!(report.malformed_line_count, 1);
}
