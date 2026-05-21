use std::process::Command;

#[test]
fn sprint74_cli_help_and_local_only_guards_are_present() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");
    let expected = [
        ("real-evidence-followup", "market-data-only"),
        ("real-evidence-attach", "local-only"),
        ("kis-real-evidence-validate", "research-only"),
        ("real-provenance-audit", "provenance"),
        ("real-preflight-audit", "preflight"),
        ("real-outcome-readiness", "live trading"),
        ("real-sequence-readiness", "training"),
        ("real-modelops-impact", "live inference"),
        ("control-tower-warning-reduce", "warning"),
        ("direct-watch-warning-rationale", "monitoring-only"),
        ("real-evidence-runbook", "copyable"),
    ];
    for (command, text) in expected {
        let help = Command::new(bin)
            .args([command, "--help"])
            .output()
            .expect("help");
        assert!(help.status.success(), "{command} --help failed");
        let stdout = String::from_utf8(help.stdout).expect("stdout");
        assert!(stdout.contains("--config"));
        assert!(stdout.to_lowercase().contains(&text.to_lowercase()));
    }
    let root_help = Command::new(bin).arg("--help").output().expect("root help");
    assert!(root_help.status.success());
    let root_stdout = String::from_utf8(root_help.stdout).expect("stdout");
    assert!(root_stdout.contains("real-evidence-followup"));
    assert!(root_stdout.contains("real-evidence-runbook"));
    assert!(!root_stdout.contains("train-model"));
    assert!(!root_stdout.contains("live-inference"));
    assert!(!root_stdout.contains("broker-order"));

    let remote = Command::new(bin)
        .args([
            "real-evidence-followup",
            "--config",
            "https://example.com/sprint74.toml",
        ])
        .output()
        .expect("remote config");
    assert!(!remote.status.success());
    let stderr = String::from_utf8(remote.stderr).expect("stderr");
    assert!(stderr.contains("must be local"));
}
