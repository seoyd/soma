#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Serialize, de::DeserializeOwned};
use soma_zero::{ModelOpsReviewClosureConfig, PredictionHistoryPackConfig};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn example_path(name: &str) -> PathBuf {
    project_root().join("examples").join(name)
}

pub fn output_dir(name: &str) -> PathBuf {
    let path = project_root()
        .join("target")
        .join("sprint66-tests")
        .join(name);
    fs::create_dir_all(&path).expect("create sprint66 output dir");
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

pub fn closure_config_from_example(name: &str, output_name: &str) -> ModelOpsReviewClosureConfig {
    let mut config = ModelOpsReviewClosureConfig::from_toml_path(&example_path(name))
        .expect("parse sprint66 closure config");
    absolutize_closure_paths(&mut config);
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn prediction_history_config_from_example(
    name: &str,
    output_name: &str,
) -> PredictionHistoryPackConfig {
    let mut config = PredictionHistoryPackConfig::from_toml_path(&example_path(name))
        .expect("parse sprint66 prediction history config");
    absolutize_history_paths(&mut config);
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn write_support_json<T: Serialize>(output_name: &str, file_name: &str, value: &T) -> String {
    let dir = output_dir(output_name);
    let path = dir.join(file_name);
    let text = serde_json::to_string_pretty(value).expect("serialize sprint66 support json");
    fs::write(&path, text).expect("write sprint66 support json");
    path.display().to_string()
}

pub fn read_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> T {
    serde_json::from_str(&fs::read_to_string(path).expect("read sprint66 support json"))
        .expect("parse sprint66 support json")
}

fn absolutize_closure_paths(config: &mut ModelOpsReviewClosureConfig) {
    for paths in [
        &mut config.external_model_research_ops_paths,
        &mut config.model_review_queue_paths,
        &mut config.owner_model_review_action_paths,
        &mut config.watchlist_paths,
        &mut config.model_risk_profile_paths,
        &mut config.leaderboard_changelog_paths,
        &mut config.prediction_history_pack_paths,
        &mut config.model_ops_baseline_paths,
        &mut config.model_ops_current_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
}

fn absolutize_history_paths(config: &mut PredictionHistoryPackConfig) {
    for paths in [
        &mut config.sequence_export_manifest_paths,
        &mut config.prediction_csv_paths,
        &mut config.model_card_paths,
        &mut config.evaluation_report_paths,
        &mut config.import_report_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
}
