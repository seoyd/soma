#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use soma_zero::ExternalPredictionImportV2Config;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn example_path(name: &str) -> PathBuf {
    project_root().join("examples").join(name)
}

pub fn output_dir(name: &str) -> PathBuf {
    let path = project_root()
        .join("target")
        .join("sprint63-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint63 output dir");
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

pub fn import_config_from_example(
    name: &str,
    output_name: &str,
) -> ExternalPredictionImportV2Config {
    let mut config = ExternalPredictionImportV2Config::from_toml_path(&example_path(name))
        .expect("parse sprint63 import config");
    absolutize_import_paths(&mut config);
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn write_support_file(output_name: &str, file_name: &str, content: &str) -> String {
    let dir = output_dir(output_name);
    let path = dir.join(file_name);
    fs::write(&path, content).expect("write sprint63 support file");
    path.display().to_string()
}

fn absolutize_import_paths(config: &mut ExternalPredictionImportV2Config) {
    config.sequence_export_manifest_path = absolutize(&config.sequence_export_manifest_path)
        .display()
        .to_string();
    if let Some(path) = &mut config.dataset_csv_path {
        *path = absolutize(path).display().to_string();
    }
    if let Some(path) = &mut config.feature_schema_manifest_path {
        *path = absolutize(path).display().to_string();
    }
    if let Some(path) = &mut config.label_manifest_path {
        *path = absolutize(path).display().to_string();
    }
    for paths in [
        &mut config.prediction_csv_paths,
        &mut config.model_card_paths,
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
