#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Serialize, de::DeserializeOwned};
use soma_zero::ExternalModelResearchOpsConfig;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn example_path(name: &str) -> PathBuf {
    project_root().join("examples").join(name)
}

pub fn output_dir(name: &str) -> PathBuf {
    let path = project_root()
        .join("target")
        .join("sprint65-tests")
        .join(name);
    fs::create_dir_all(&path).expect("create sprint65 output dir");
    path
}

pub fn absolutize(path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        project_root().join(candidate)
    }
}

pub fn research_ops_config_from_example(
    name: &str,
    output_name: &str,
) -> ExternalModelResearchOpsConfig {
    let mut config = ExternalModelResearchOpsConfig::from_toml_path(&example_path(name))
        .expect("parse sprint65 research ops config");
    absolutize_research_ops_paths(&mut config);
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn write_support_file(output_name: &str, file_name: &str, content: &str) -> String {
    let dir = output_dir(output_name);
    let path = dir.join(file_name);
    fs::write(&path, content).expect("write sprint65 support file");
    path.display().to_string()
}

pub fn write_support_json<T: Serialize>(output_name: &str, file_name: &str, value: &T) -> String {
    let dir = output_dir(output_name);
    let path = dir.join(file_name);
    let text = serde_json::to_string_pretty(value).expect("serialize sprint65 support json");
    fs::write(&path, text).expect("write sprint65 support json");
    path.display().to_string()
}

pub fn read_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> T {
    serde_json::from_str(&fs::read_to_string(path).expect("read sprint65 support json"))
        .expect("parse sprint65 support json")
}

fn absolutize_research_ops_paths(config: &mut ExternalModelResearchOpsConfig) {
    for paths in [
        &mut config.external_artifact_registry_paths,
        &mut config.conservative_leaderboard_paths,
        &mut config.calibration_drift_paths,
        &mut config.evaluation_history_paths,
        &mut config.model_version_comparison_paths,
        &mut config.previous_external_comparison_paths,
        &mut config.external_registry_audit_paths,
        &mut config.owner_model_review_paths,
        &mut config.control_tower_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
}
