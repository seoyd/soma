mod common;

use std::fs;
use std::process::Command;

use soma_zero::OfficialEvidenceAcquisitionPlan;

#[test]
fn cli_help_exposes_official_acquire_as_research_only() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("run --help");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("official-acquire"));
    assert!(stdout.contains("Research-only"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  account"));
}

#[test]
fn official_acquire_rejects_remote_config_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "official-acquire",
            "--config",
            "https://example.com/acquire.toml",
        ])
        .output()
        .expect("run official-acquire");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("official-acquire config path must be local")
    );
}

#[test]
fn official_acquire_runs_with_local_config() {
    let config_path = common::output_dir("official-acquire-cli").join("acquire.toml");
    fs::write(
        &config_path,
        OfficialEvidenceAcquisitionPlan {
            run_collection: false,
            expansion_config: None,
            ..OfficialEvidenceAcquisitionPlan::default()
        }
        .to_toml_string()
        .expect("serialize"),
    )
    .expect("write config");

    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "official-acquire",
            "--config",
            &config_path.display().to_string(),
        ])
        .output()
        .expect("run official-acquire");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("plan_id="));
}
