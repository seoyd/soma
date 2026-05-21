mod common;

use std::process::Command;

#[test]
fn trace_cli_help_texts_stay_static_and_local_only() {
    let checks = [
        ("model-ops-trace", "static/read-only"),
        ("model-trace-index", "local-only"),
        ("model-decision-conflicts", "research-only"),
        ("model-regression-trace", "diagnostic"),
        ("model-qa-trace", "operator QA only"),
        ("model-action-trace", "no execution"),
        ("model-version-diff-trace", "deterministic"),
    ];
    for (command, phrase) in checks {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--help"])
            .output()
            .expect("run help");
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
        assert!(stdout.contains(phrase), "missing help phrase for {command}");
        for forbidden in ["live trading readiness", "broker account control"] {
            assert!(
                !stdout.contains(forbidden),
                "unexpected help phrase for {command}"
            );
        }
    }
}
