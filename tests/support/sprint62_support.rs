#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use soma_zero::{SequenceDatasetDriftGuardConfig, SequenceDatasetExportConfig};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn example_path(name: &str) -> PathBuf {
    project_root().join("examples").join(name)
}

pub fn output_dir(name: &str) -> PathBuf {
    let path = project_root()
        .join("target")
        .join("sprint62-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint62 output dir");
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

pub fn export_config_from_example(name: &str, output_name: &str) -> SequenceDatasetExportConfig {
    let mut config = SequenceDatasetExportConfig::from_toml_path(&example_path(name))
        .expect("parse export config");
    absolutize_export_paths(&mut config);
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn drift_config_from_example(name: &str, output_name: &str) -> SequenceDatasetDriftGuardConfig {
    let mut config = SequenceDatasetDriftGuardConfig::from_toml_path(&example_path(name))
        .expect("parse drift config");
    absolutize_drift_paths(&mut config);
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

fn absolutize_export_paths(config: &mut SequenceDatasetExportConfig) {
    for paths in [
        &mut config.sequence_readiness_hardening_paths,
        &mut config.kis_evidence_closure_paths,
        &mut config.outcome_link_depth_closure_paths,
        &mut config.kis_canonical_csv_paths,
        &mut config.feature_schema_lock_paths,
        &mut config.label_alignment_audit_paths,
        &mut config.no_lookahead_proof_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
}

fn absolutize_drift_paths(config: &mut SequenceDatasetDriftGuardConfig) {
    if let Some(path) = &mut config.baseline_manifest_path {
        *path = absolutize(path).display().to_string();
    }
    config.current_manifest_path = absolutize(&config.current_manifest_path)
        .display()
        .to_string();
}
