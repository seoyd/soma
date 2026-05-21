mod common;

use std::process::Command;

#[test]
fn sprint69_trace_coverage_cli_help_texts_stay_static_and_local_only() {
    let checks = [
        ("baseline-snapshot-coverage", "static/read-only"),
        ("comparison-target-registry", "research-only"),
        ("missing-comparison-targets", "diagnostic"),
        ("trace-completeness-audit", "coverage audit"),
        ("downgrade-evidence-audit", "conservative"),
        ("snapshot-diff-integrity", "deterministic"),
        ("control-tower-trace-coverage", "read-only"),
    ];
    for (command, phrase) in checks {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--help"])
            .output()
            .expect("run help");
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
        assert!(stdout.contains(phrase), "missing help phrase for {command}");
        for forbidden in [
            "live trading readiness",
            "broker account control",
            "train button",
        ] {
            assert!(
                !stdout.contains(forbidden),
                "unexpected help phrase for {command}"
            );
        }
    }
}
