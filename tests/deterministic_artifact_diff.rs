mod common;
#[path = "support/sprint59_support.rs"]
mod sprint59_support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{
    ArtifactDiffStatus, DeterministicArtifactDiffConfig, run_deterministic_artifact_diff,
};

#[test]
fn identical_artifacts_after_ignored_fields_report_no_diff() {
    let mut config = DeterministicArtifactDiffConfig::from_toml_path(
        &sprint59_support::example_path("soma_system_benchmark_diff.toml"),
    )
    .expect("parse diff config");
    config.baseline_artifact_paths = config
        .baseline_artifact_paths
        .into_iter()
        .map(|path| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(path)
                .display()
                .to_string()
        })
        .collect();
    config.current_artifact_paths = config
        .current_artifact_paths
        .into_iter()
        .map(|path| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(path)
                .display()
                .to_string()
        })
        .collect();
    config.output_root = sprint59_support::output_dir("artifact-diff-no-diff")
        .display()
        .to_string();
    let report = run_deterministic_artifact_diff(&config).expect("run no-diff report");
    assert_eq!(report.diff_status, ArtifactDiffStatus::NoDiff);
}

#[test]
fn changed_payload_without_allowance_is_unexpected_diff() {
    let output_dir = sprint59_support::output_dir("artifact-diff-unexpected");
    let baseline = output_dir.join("baseline.json");
    let current = output_dir.join("current.json");
    fs::write(
        &baseline,
        "{ \"artifact_id\": \"sample\", \"status\": \"stable\" }",
    )
    .expect("write baseline");
    fs::write(
        &current,
        "{ \"artifact_id\": \"sample\", \"status\": \"changed\" }",
    )
    .expect("write current");
    let report = run_deterministic_artifact_diff(&DeterministicArtifactDiffConfig {
        diff_id: "unexpected-diff".to_string(),
        baseline_artifact_paths: vec![baseline.display().to_string()],
        current_artifact_paths: vec![current.display().to_string()],
        output_root: output_dir.display().to_string(),
        ..DeterministicArtifactDiffConfig::default()
    })
    .expect("run unexpected diff report");
    assert_eq!(report.diff_status, ArtifactDiffStatus::UnexpectedDiff);
}
