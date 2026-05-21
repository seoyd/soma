#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use soma_zero::ExternalModelArtifactRegistryConfig;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn example_path(name: &str) -> PathBuf {
    project_root().join("examples").join(name)
}

pub fn output_dir(name: &str) -> PathBuf {
    let path = project_root()
        .join("target")
        .join("sprint64-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint64 output dir");
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

pub fn registry_config_from_example(
    name: &str,
    output_name: &str,
) -> ExternalModelArtifactRegistryConfig {
    let mut config = ExternalModelArtifactRegistryConfig::from_toml_path(&example_path(name))
        .expect("parse sprint64 registry config");
    absolutize_registry_paths(&mut config);
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn write_support_file(output_name: &str, file_name: &str, content: &str) -> String {
    let dir = output_dir(output_name);
    let path = dir.join(file_name);
    fs::write(&path, content).expect("write sprint64 support file");
    path.display().to_string()
}

pub fn write_support_json<T: Serialize>(output_name: &str, file_name: &str, value: &T) -> String {
    let dir = output_dir(output_name);
    let path = dir.join(file_name);
    let text = serde_json::to_string_pretty(value).expect("serialize sprint64 support json");
    fs::write(&path, text).expect("write sprint64 support json");
    path.display().to_string()
}

fn absolutize_registry_paths(config: &mut ExternalModelArtifactRegistryConfig) {
    for paths in [
        &mut config.sequence_export_manifest_paths,
        &mut config.feature_schema_manifest_paths,
        &mut config.label_manifest_paths,
        &mut config.external_prediction_import_report_paths,
        &mut config.external_model_card_paths,
        &mut config.external_model_evaluation_report_paths,
        &mut config.external_vs_trinity_report_paths,
        &mut config.external_prediction_ablation_report_paths,
        &mut config.external_model_promotion_gate_paths,
        &mut config.mamba3fin_contract_paths,
        &mut config.baseline_reference_paths,
        &mut config.trinity_reference_paths,
        &mut config.no_trade_reference_paths,
        &mut config.risk_denied_reference_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
}
