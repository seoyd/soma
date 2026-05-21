#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use soma_zero::{
    KISEvidenceClosureConfig, KISEvidenceExpansionPlanV2Config, OutcomeLinkDepthClosureV2Config,
    OwnerReviewDisciplineV2Config, SequenceDatasetPreparationConfig,
};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn example_path(name: &str) -> PathBuf {
    project_root().join("examples").join(name)
}

pub fn output_dir(name: &str) -> PathBuf {
    let path = project_root()
        .join("target")
        .join("sprint61-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint61 output dir");
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

pub fn plan_config_from_example(name: &str, output_name: &str) -> KISEvidenceExpansionPlanV2Config {
    let mut config = KISEvidenceExpansionPlanV2Config::from_toml_path(&example_path(name))
        .expect("parse plan config");
    absolutize_plan_paths(&mut config);
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn closure_config_from_example(name: &str, output_name: &str) -> KISEvidenceClosureConfig {
    let mut config = KISEvidenceClosureConfig::from_toml_path(&example_path(name))
        .expect("parse closure config");
    absolutize_closure_paths(&mut config);
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn outcome_config_from_example(
    name: &str,
    output_name: &str,
) -> OutcomeLinkDepthClosureV2Config {
    let mut config = OutcomeLinkDepthClosureV2Config::from_toml_path(&example_path(name))
        .expect("parse outcome config");
    absolutize_outcome_paths(&mut config);
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn discipline_config_from_example(
    name: &str,
    output_name: &str,
) -> OwnerReviewDisciplineV2Config {
    let mut config = OwnerReviewDisciplineV2Config::from_toml_path(&example_path(name))
        .expect("parse discipline config");
    absolutize_discipline_paths(&mut config);
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn sequence_config_from_example(
    name: &str,
    output_name: &str,
) -> SequenceDatasetPreparationConfig {
    let mut config = SequenceDatasetPreparationConfig::from_toml_path(&example_path(name))
        .expect("parse sequence config");
    absolutize_sequence_paths(&mut config);
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

fn absolutize_plan_paths(config: &mut KISEvidenceExpansionPlanV2Config) {
    for paths in [
        &mut config.kis_evidence_depth_report_paths,
        &mut config.kis_smoke_report_paths,
        &mut config.kis_activation_report_paths,
        &mut config.kis_collection_plan_paths,
        &mut config.kis_canonical_csv_paths,
        &mut config.kis_provenance_paths,
        &mut config.kis_preflight_paths,
        &mut config.outcome_link_closure_paths,
        &mut config.counterfactual_completion_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
}

fn absolutize_closure_paths(config: &mut KISEvidenceClosureConfig) {
    if let Some(path) = &mut config.expansion_plan_config_path {
        *path = absolutize(path).display().to_string();
    }
    if let Some(path) = &mut config.evidence_hardening_config_path {
        *path = absolutize(path).display().to_string();
    }
    if let Some(path) = &mut config.control_tower_refresh_config_path {
        *path = absolutize(path).display().to_string();
    }
    for paths in [
        &mut config.outcome_link_coverage_paths,
        &mut config.counterfactual_coverage_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
}

fn absolutize_outcome_paths(config: &mut OutcomeLinkDepthClosureV2Config) {
    for paths in [
        &mut config.outcome_link_coverage_paths,
        &mut config.kis_canonical_csv_paths,
        &mut config.barrier_profile_registry_paths,
        &mut config.complete_row_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
}

fn absolutize_discipline_paths(config: &mut OwnerReviewDisciplineV2Config) {
    for paths in [
        &mut config.owner_review_queue_paths,
        &mut config.owner_input_paths,
        &mut config.owner_impact_report_paths,
        &mut config.human_confirm_paths,
        &mut config.candidate_queue_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
}

fn absolutize_sequence_paths(config: &mut SequenceDatasetPreparationConfig) {
    for paths in [
        &mut config.kis_evidence_closure_paths,
        &mut config.complete_row_paths,
        &mut config.kis_canonical_csv_paths,
        &mut config.feature_schema_paths,
        &mut config.outcome_link_paths,
        &mut config.counterfactual_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
}
