mod common;

use std::process::Command;

use soma_zero::{ComparableCommitteeEvidenceConfig, CounterfactualDepthClosureConfig};

#[test]
fn counterfactual_depth_cli_help_lists_new_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("comparable-evidence"));
    assert!(stdout.contains("counterfactual-depth-plan"));
    assert!(stdout.contains("counterfactual-depth-close"));
}

#[test]
fn counterfactual_depth_commands_reject_remote_paths() {
    for (command, expected) in [
        (
            "comparable-evidence",
            "comparable-evidence config path must be local",
        ),
        (
            "counterfactual-depth-plan",
            "counterfactual-depth-plan config path must be local",
        ),
        (
            "counterfactual-depth-close",
            "counterfactual-depth-close config path must be local",
        ),
        (
            "scorecard-rerun",
            "scorecard-rerun config path must be local",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("remote");
        assert!(String::from_utf8_lossy(&output.stderr).contains(expected));
    }
}

#[test]
fn comparable_evidence_and_closure_commands_run_with_local_configs() {
    let comparable = ComparableCommitteeEvidenceConfig {
        comparable_id: "cli-comparable".to_string(),
        output_root: common::output_dir("cli-comparable-root")
            .display()
            .to_string(),
        ..ComparableCommitteeEvidenceConfig::default()
    };
    let comparable_path = common::output_dir("cli-comparable-config").join("comparable.toml");
    std::fs::write(&comparable_path, comparable.to_toml_string().expect("toml"))
        .expect("write comparable");

    let comparable_run = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "comparable-evidence",
            "--config",
            &comparable_path.display().to_string(),
        ])
        .output()
        .expect("comparable run");
    assert!(comparable_run.status.success());
    assert!(String::from_utf8_lossy(&comparable_run.stdout).contains("research_only_warning"));

    let closure = CounterfactualDepthClosureConfig {
        closure_id: "cli-closure".to_string(),
        comparable_evidence_config_path: Some(comparable_path.display().to_string()),
        output_root: common::output_dir("cli-closure-root").display().to_string(),
        max_build_attempts: 2,
        ..CounterfactualDepthClosureConfig::default()
    };
    let closure_path = common::output_dir("cli-closure-config").join("closure.toml");
    std::fs::write(&closure_path, closure.to_toml_string().expect("toml")).expect("write closure");

    let closure_run = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "counterfactual-depth-close",
            "--config",
            &closure_path.display().to_string(),
        ])
        .output()
        .expect("closure run");
    assert!(closure_run.status.success());
    assert!(String::from_utf8_lossy(&closure_run.stdout).contains("research_only_warning"));
}
