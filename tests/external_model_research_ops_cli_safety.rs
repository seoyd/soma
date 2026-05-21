use std::process::Command;

#[test]
fn sprint65_help_texts_include_safety_language() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");

    let research_ops = Command::new(bin)
        .args(["external-model-research-ops", "--help"])
        .output()
        .expect("run research ops help");
    assert!(String::from_utf8_lossy(&research_ops.stdout).contains("no training"));

    let review_queue = Command::new(bin)
        .args(["external-model-review-queue", "--help"])
        .output()
        .expect("run review queue help");
    assert!(
        String::from_utf8_lossy(&review_queue.stdout)
            .to_ascii_lowercase()
            .contains("research-only")
    );

    let watchlist = Command::new(bin)
        .args(["external-model-watchlist", "--help"])
        .output()
        .expect("run watchlist help");
    assert!(
        String::from_utf8_lossy(&watchlist.stdout)
            .to_ascii_lowercase()
            .contains("offline-only")
    );

    let comparability = Command::new(bin)
        .args(["model-comparability-matrix", "--help"])
        .output()
        .expect("run comparability help");
    assert!(
        String::from_utf8_lossy(&comparability.stdout)
            .to_ascii_lowercase()
            .contains("diagnostic-only")
    );

    let completeness = Command::new(bin)
        .args(["artifact-completeness", "--help"])
        .output()
        .expect("run completeness help");
    assert!(
        String::from_utf8_lossy(&completeness.stdout)
            .to_ascii_lowercase()
            .contains("local-only")
    );

    let risk = Command::new(bin)
        .args(["model-risk-profile", "--help"])
        .output()
        .expect("run risk help");
    assert!(
        String::from_utf8_lossy(&risk.stdout)
            .to_ascii_lowercase()
            .contains("no live promotion")
    );

    let changelog = Command::new(bin)
        .args(["model-leaderboard-changelog", "--help"])
        .output()
        .expect("run changelog help");
    assert!(
        String::from_utf8_lossy(&changelog.stdout)
            .to_ascii_lowercase()
            .contains("no deployment")
    );
}

#[test]
fn forbidden_runtime_commands_do_not_exist() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");
    for command in [
        "train-external-model",
        "live-inference",
        "mamba-runtime",
        "broker-order-account",
    ] {
        let output = Command::new(bin)
            .args([command, "--help"])
            .output()
            .expect("run forbidden help");
        assert!(!output.status.success());
    }
}

#[test]
fn remote_paths_are_rejected_by_cli() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");
    let output = Command::new(bin)
        .args([
            "external-model-research-ops",
            "--config",
            "https://example.com/remote.toml",
        ])
        .output()
        .expect("run remote config");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("external-model-research-ops config path must be local")
    );
}
