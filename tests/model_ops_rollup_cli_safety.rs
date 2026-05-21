mod common;

use std::process::Command;

#[test]
fn rollup_cli_help_texts_stay_offline_and_read_only() {
    let checks = [
        ("model-ops-rollup", "no-training"),
        ("model-regression-explain", "offline-only"),
        ("operator-qa-rollup", "research-only"),
        ("decision-log-rollup", "diagnostic"),
        ("model-risk-rollup", "no live promotion"),
        ("model-action-priority", "no execution"),
        ("control-tower-model-ops-rollup", "read-only"),
    ];
    for (command, phrase) in checks {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--help"])
            .output()
            .expect("run help");
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
        assert!(stdout.contains(phrase), "missing help phrase for {command}");
        for forbidden in ["order account", "live trading readiness"] {
            assert!(
                !stdout.contains(forbidden),
                "unexpected help phrase for {command}"
            );
        }
    }
}
