#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Serialize, de::DeserializeOwned};
use soma_zero::{ModelOpsTraceBundle, ModelOpsTraceConfig, ModelOpsTraceRunner};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn example_path(name: &str) -> PathBuf {
    project_root().join("examples").join(name)
}

pub fn output_dir(name: &str) -> PathBuf {
    let path = project_root()
        .join("target")
        .join("sprint68-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint68 output dir");
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

pub fn trace_config_from_example(name: &str, output_name: &str) -> ModelOpsTraceConfig {
    let mut config =
        ModelOpsTraceConfig::from_toml_path(&example_path(name)).expect("parse sprint68 config");
    absolutize_trace_paths(&mut config);
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn run_trace(name: &str, output_name: &str) -> ModelOpsTraceBundle {
    let config = trace_config_from_example(name, output_name);
    ModelOpsTraceRunner::default()
        .run(&config)
        .expect("run sprint68 trace")
}

pub fn write_support_json<T: Serialize>(output_name: &str, file_name: &str, value: &T) -> String {
    let dir = output_dir(output_name);
    let path = dir.join(file_name);
    let text = serde_json::to_string_pretty(value).expect("serialize sprint68 support json");
    fs::write(&path, text).expect("write sprint68 support json");
    path.display().to_string()
}

pub fn read_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> T {
    serde_json::from_str(&fs::read_to_string(path).expect("read sprint68 support json"))
        .expect("parse sprint68 support json")
}

fn absolutize_trace_paths(config: &mut ModelOpsTraceConfig) {
    for paths in [
        &mut config.model_ops_rollup_paths,
        &mut config.model_version_summary_card_paths,
        &mut config.regression_explanation_paths,
        &mut config.operator_qa_rollup_paths,
        &mut config.decision_log_rollup_paths,
        &mut config.model_ops_risk_rollup_paths,
        &mut config.model_ops_action_priority_paths,
        &mut config.external_artifact_registry_paths,
        &mut config.external_model_research_ops_paths,
        &mut config.conservative_leaderboard_paths,
        &mut config.prediction_history_pack_paths,
        &mut config.model_ops_regression_guard_paths,
        &mut config.baseline_artifact_paths,
        &mut config.current_artifact_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
}
