#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Serialize, de::DeserializeOwned};
use soma_zero::{ModelOpsRollupConfig, ModelOpsRollupRunner};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn example_path(name: &str) -> PathBuf {
    project_root().join("examples").join(name)
}

pub fn output_dir(name: &str) -> PathBuf {
    let path = project_root()
        .join("target")
        .join("sprint67-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint67 output dir");
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

pub fn rollup_config_from_example(name: &str, output_name: &str) -> ModelOpsRollupConfig {
    let mut config =
        ModelOpsRollupConfig::from_toml_path(&example_path(name)).expect("parse sprint67 config");
    absolutize_rollup_paths(&mut config);
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn run_rollup(name: &str, output_name: &str) -> soma_zero::ModelOpsRollupBundle {
    let config = rollup_config_from_example(name, output_name);
    ModelOpsRollupRunner::default()
        .run(&config)
        .expect("run sprint67 rollup")
}

pub fn write_support_json<T: Serialize>(output_name: &str, file_name: &str, value: &T) -> String {
    let dir = output_dir(output_name);
    let path = dir.join(file_name);
    let text = serde_json::to_string_pretty(value).expect("serialize sprint67 support json");
    fs::write(&path, text).expect("write sprint67 support json");
    path.display().to_string()
}

pub fn read_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> T {
    serde_json::from_str(&fs::read_to_string(path).expect("read sprint67 support json"))
        .expect("parse sprint67 support json")
}

fn absolutize_rollup_paths(config: &mut ModelOpsRollupConfig) {
    for paths in [
        &mut config.model_review_closure_paths,
        &mut config.prediction_history_pack_paths,
        &mut config.model_ops_decision_log_paths,
        &mut config.model_ops_operator_qa_paths,
        &mut config.model_ops_regression_guard_paths,
        &mut config.control_tower_model_ops_refresh_paths,
        &mut config.external_model_research_ops_paths,
        &mut config.conservative_leaderboard_paths,
        &mut config.calibration_drift_paths,
        &mut config.model_risk_profile_paths,
        &mut config.owner_model_review_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
}
