use std::process::Command;

#[test]
fn offline_evidence_attachment_cli_help_and_guards_are_present() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");
    for command in [
        "offline-evidence-attach",
        "prediction-history-expand",
        "retirement-regression-pack",
        "evidence-gap-close-v2",
        "owner-checklist-close",
        "direct-watch-score",
        "briefing-readiness-gate",
    ] {
        let help = Command::new(bin)
            .args([command, "--help"])
            .output()
            .expect("sprint72 help");
        assert!(help.status.success(), "{command} --help failed");
        let stdout = String::from_utf8(help.stdout).expect("stdout utf8");
        assert!(
            stdout.contains("--config"),
            "{command} help missing --config"
        );
    }

    let root_help = Command::new(bin)
        .arg("--help")
        .output()
        .expect("root --help");
    assert!(root_help.status.success());
    let root_stdout = String::from_utf8(root_help.stdout).expect("stdout utf8");
    assert!(root_stdout.contains("offline-evidence-attach"));
    assert!(root_stdout.contains("prediction-history-expand"));
    assert!(root_stdout.contains("briefing-readiness-gate"));
    assert!(!root_stdout.contains("train-model"));
    assert!(!root_stdout.contains("live-inference"));
    assert!(!root_stdout.contains("broker-order"));

    let remote = Command::new(bin)
        .args([
            "offline-evidence-attach",
            "--config",
            "https://example.com/offline.toml",
        ])
        .output()
        .expect("remote config");
    assert!(!remote.status.success());
    let stderr = String::from_utf8(remote.stderr).expect("stderr utf8");
    assert!(stderr.contains("must be local") || stderr.contains("config path must be local"));
}
