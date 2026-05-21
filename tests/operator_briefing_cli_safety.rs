use std::process::Command;

#[test]
fn operator_briefing_cli_help_and_local_only_guards_are_present() {
    let help = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--bin",
            "soma_experiment",
            "--",
            "operator-briefing",
            "--help",
        ])
        .output()
        .expect("operator-briefing --help");
    assert!(help.status.success());
    let stdout = String::from_utf8(help.stdout).expect("stdout utf8");
    assert!(stdout.contains("static/read-only"));
    assert!(stdout.contains("paper-only"));

    let owner_help = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--bin",
            "soma_experiment",
            "--",
            "owner-action-checklist",
            "--help",
        ])
        .output()
        .expect("owner-action-checklist --help");
    assert!(owner_help.status.success());
    let owner_stdout = String::from_utf8(owner_help.stdout).expect("stdout utf8");
    assert!(owner_stdout.contains("paper-only"));

    let queue_help = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--bin",
            "soma_experiment",
            "--",
            "operator-decision-queue",
            "--help",
        ])
        .output()
        .expect("operator-decision-queue --help");
    assert!(queue_help.status.success());
    let queue_stdout = String::from_utf8(queue_help.stdout).expect("stdout utf8");
    assert!(queue_stdout.contains("no execution controls"));

    let root_help = Command::new("cargo")
        .args(["run", "--quiet", "--bin", "soma_experiment", "--", "--help"])
        .output()
        .expect("root --help");
    assert!(root_help.status.success());
    let root_stdout = String::from_utf8(root_help.stdout).expect("stdout utf8");
    assert!(!root_stdout.contains("train-model"));
    assert!(!root_stdout.contains("live-inference"));
    assert!(!root_stdout.contains("mamba-runtime"));

    let remote = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--bin",
            "soma_experiment",
            "--",
            "operator-briefing",
            "--config",
            "https://example.com/operator.toml",
        ])
        .output()
        .expect("remote config");
    assert!(!remote.status.success());
    let stderr = String::from_utf8(remote.stderr).expect("stderr utf8");
    assert!(stderr.contains("must be local") || stderr.contains("config path must be local"));
}
