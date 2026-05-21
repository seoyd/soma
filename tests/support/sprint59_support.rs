#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use soma_zero::{DeterministicArtifactDiffConfig, SystemIntegrationReviewConfig};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn example_path(name: &str) -> PathBuf {
    project_root().join("examples").join(name)
}

pub fn output_dir(name: &str) -> PathBuf {
    let path = project_root()
        .join("target")
        .join("sprint59-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint59 output dir");
    path
}

pub fn review_config_from_example(name: &str, output_name: &str) -> SystemIntegrationReviewConfig {
    let mut config = SystemIntegrationReviewConfig::from_toml_path(&example_path(name))
        .expect("parse review config");
    absolutize_review_paths(&mut config);
    let root = output_dir(output_name);
    config.output_root = root.display().to_string();
    if let Some(path) = config.artifact_diff_config_path.clone() {
        let absolute_path = absolutize(&path);
        let mut diff_config = DeterministicArtifactDiffConfig::from_toml_path(&absolute_path)
            .expect("parse diff config");
        absolutize_diff_paths(&mut diff_config);
        diff_config.output_root = root.join("diff").display().to_string();
        let diff_config_path = root.join("system_benchmark_diff.toml");
        fs::write(
            &diff_config_path,
            toml::to_string_pretty(&diff_config).expect("serialize diff config"),
        )
        .expect("write diff config");
        config.artifact_diff_config_path = Some(diff_config_path.display().to_string());
    }
    config
}

fn absolutize_review_paths(config: &mut SystemIntegrationReviewConfig) {
    for paths in [
        &mut config.core_check_report_paths,
        &mut config.core_completion_audit_paths,
        &mut config.core_scorecard_paths,
        &mut config.kis_smoke_report_paths,
        &mut config.kis_evidence_depth_report_paths,
        &mut config.control_tower_state_paths,
        &mut config.control_tower_refresh_paths,
        &mut config.trinity_loop_report_paths,
        &mut config.committee_cycle_report_paths,
        &mut config.chair_report_paths,
        &mut config.risk_report_paths,
        &mut config.owner_review_queue_paths,
        &mut config.paper_lifecycle_paths,
        &mut config.operational_runbook_paths,
        &mut config.dashboard_html_paths,
        &mut config.dashboard_json_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
}

fn absolutize_diff_paths(config: &mut DeterministicArtifactDiffConfig) {
    for paths in [
        &mut config.baseline_artifact_paths,
        &mut config.current_artifact_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
}

fn absolutize(path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        project_root().join(candidate)
    }
}
