#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use soma_zero::EvidenceHardeningConfig;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn example_path(name: &str) -> PathBuf {
    project_root().join("examples").join(name)
}

pub fn output_dir(name: &str) -> PathBuf {
    let path = project_root()
        .join("target")
        .join("sprint60-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint60 output dir");
    path
}

pub fn config_from_example(name: &str, output_name: &str) -> EvidenceHardeningConfig {
    let mut config = EvidenceHardeningConfig::from_toml_path(&example_path(name))
        .expect("parse sprint60 config");
    absolutize_config_paths(&mut config);
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn write_support_json(output_name: &str, file_name: &str, value: &Value) -> String {
    let dir = output_dir(output_name);
    let path = dir.join(file_name);
    fs::write(
        &path,
        serde_json::to_string_pretty(value).expect("serialize support json"),
    )
    .expect("write support json");
    path.display().to_string()
}

pub fn absolutize(path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        project_root().join(candidate)
    }
}

fn absolutize_config_paths(config: &mut EvidenceHardeningConfig) {
    for paths in [
        &mut config.kis_smoke_report_paths,
        &mut config.system_review_bundle_paths,
        &mut config.control_tower_state_paths,
        &mut config.control_tower_html_paths,
        &mut config.owner_review_queue_paths,
        &mut config.sequence_readiness_report_paths,
        &mut config.mamba_readiness_snapshot_paths,
        &mut config.operational_runbook_paths,
        &mut config.supporting_artifact_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
}
