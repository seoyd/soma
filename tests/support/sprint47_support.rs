#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use soma_zero::{
    BatchCounterfactualCompletionConfig, BatchOutcomeLinkageV3Config, FutureWindowScaleOutConfig,
    MultiRowOfficialEvidenceSetConfig, OfficialEvidenceScaleOutConfig,
    OfficialEvidenceSufficiencyV2Config,
};

pub fn repo_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

pub fn output_dir(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("sprint47-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create output dir");
    path
}

pub fn example_multi_row_set(name: &str, example: &str) -> MultiRowOfficialEvidenceSetConfig {
    let mut config = MultiRowOfficialEvidenceSetConfig::from_toml_path(&repo_path(example))
        .expect("multi-row config");
    config.output_root = output_dir(name).display().to_string();
    config
}

pub fn example_future_window(name: &str) -> FutureWindowScaleOutConfig {
    let mut config = FutureWindowScaleOutConfig::from_toml_path(&repo_path(
        "examples/soma_future_window_scaleout_multi_row.toml",
    ))
    .expect("future-window config");
    config.output_root = output_dir(name).display().to_string();
    config
}

pub fn example_batch_outcome(name: &str) -> BatchOutcomeLinkageV3Config {
    let mut config = BatchOutcomeLinkageV3Config::from_toml_path(&repo_path(
        "examples/soma_batch_outcome_linkage_v3_multi_row.toml",
    ))
    .expect("batch outcome config");
    config.output_root = output_dir(name).display().to_string();
    config
}

pub fn example_batch_counterfactual(name: &str) -> BatchCounterfactualCompletionConfig {
    let mut config = BatchCounterfactualCompletionConfig::from_toml_path(&repo_path(
        "examples/soma_batch_counterfactual_complete_multi_row.toml",
    ))
    .expect("batch counterfactual config");
    config.output_root = output_dir(name).display().to_string();
    config
}

pub fn example_sufficiency(name: &str, example: &str) -> OfficialEvidenceSufficiencyV2Config {
    let mut config = OfficialEvidenceSufficiencyV2Config::from_toml_path(&repo_path(example))
        .expect("sufficiency config");
    config.output_root = output_dir(name).display().to_string();
    config
}

pub fn write_multi_row_config(
    path: &Path,
    mut config: MultiRowOfficialEvidenceSetConfig,
) -> PathBuf {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create config dir");
    }
    config.output_root = path.parent().unwrap().join("outputs").display().to_string();
    fs::write(path, config.to_toml_string().expect("multi-row toml"))
        .expect("write multi-row toml");
    path.to_path_buf()
}

pub fn write_batch_outcome_config(path: &Path, mut config: BatchOutcomeLinkageV3Config) -> PathBuf {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create config dir");
    }
    config.output_root = path.parent().unwrap().join("outputs").display().to_string();
    fs::write(path, config.to_toml_string().expect("batch outcome toml"))
        .expect("write batch outcome toml");
    path.to_path_buf()
}

pub fn write_batch_counterfactual_config(
    path: &Path,
    mut config: BatchCounterfactualCompletionConfig,
) -> PathBuf {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create config dir");
    }
    config.output_root = path.parent().unwrap().join("outputs").display().to_string();
    fs::write(
        path,
        config.to_toml_string().expect("batch counterfactual toml"),
    )
    .expect("write batch counterfactual toml");
    path.to_path_buf()
}

pub fn write_future_window_config(path: &Path, mut config: FutureWindowScaleOutConfig) -> PathBuf {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create config dir");
    }
    config.output_root = path.parent().unwrap().join("outputs").display().to_string();
    fs::write(path, config.to_toml_string().expect("future-window toml"))
        .expect("write future-window toml");
    path.to_path_buf()
}

pub fn write_sufficiency_config(
    path: &Path,
    mut config: OfficialEvidenceSufficiencyV2Config,
) -> PathBuf {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create config dir");
    }
    config.output_root = path.parent().unwrap().join("outputs").display().to_string();
    fs::write(path, config.to_toml_string().expect("sufficiency toml"))
        .expect("write sufficiency toml");
    path.to_path_buf()
}

pub fn scaleout_config(name: &str) -> OfficialEvidenceScaleOutConfig {
    let root = output_dir(name);
    let config_dir = root.join("configs");
    fs::create_dir_all(&config_dir).expect("config dir");

    let multi = write_multi_row_config(
        &config_dir.join("multi_row_set.toml"),
        example_multi_row_set(
            &format!("{name}-multi"),
            "examples/soma_multi_row_official_set_multi_row.toml",
        ),
    );
    let before_multi = write_multi_row_config(
        &config_dir.join("single_row_set.toml"),
        example_multi_row_set(
            &format!("{name}-single"),
            "examples/soma_multi_row_official_set_single_row.toml",
        ),
    );
    let future = write_future_window_config(
        &config_dir.join("future_window.toml"),
        example_future_window(&format!("{name}-future")),
    );
    let batch_outcome = write_batch_outcome_config(
        &config_dir.join("batch_outcome.toml"),
        example_batch_outcome(&format!("{name}-batch-outcome")),
    );
    let batch_counterfactual = write_batch_counterfactual_config(
        &config_dir.join("batch_counterfactual.toml"),
        example_batch_counterfactual(&format!("{name}-batch-counterfactual")),
    );
    let mut sufficiency = example_sufficiency(
        &format!("{name}-sufficiency"),
        "examples/soma_official_evidence_sufficiency_v2_multi_row.toml",
    );
    sufficiency.multi_row_set_path = multi.display().to_string();
    sufficiency.batch_outcome_linkage_path = Some(batch_outcome.display().to_string());
    sufficiency.batch_counterfactual_completion_path =
        Some(batch_counterfactual.display().to_string());
    let sufficiency_path =
        write_sufficiency_config(&config_dir.join("sufficiency.toml"), sufficiency);

    OfficialEvidenceScaleOutConfig {
        scaleout_id: format!("{name}-scaleout"),
        multi_row_set_config_path: multi.display().to_string(),
        before_multi_row_set_config_path: Some(before_multi.display().to_string()),
        future_window_scaleout_config_path: Some(future.display().to_string()),
        batch_outcome_linkage_config_path: Some(batch_outcome.display().to_string()),
        batch_counterfactual_completion_config_path: Some(
            batch_counterfactual.display().to_string(),
        ),
        before_batch_outcome_linkage_config_path: None,
        before_batch_counterfactual_completion_config_path: None,
        official_evidence_sufficiency_v2_config_path: Some(sufficiency_path.display().to_string()),
        committee_official_benchmark_config_path: None,
        outcome_coverage_config_path: None,
        counterfactual_depth_closure_config_path: None,
        core_performance_config_path: None,
        output_root: root.join("scaleout").display().to_string(),
        run_multi_row_set: true,
        run_future_window_scaleout: true,
        run_batch_outcome_linkage: true,
        run_batch_counterfactual_completion: true,
        run_complete_row_rebuild: true,
        run_committee_official_benchmark: true,
        run_outcome_coverage: true,
        run_counterfactual_depth_close: true,
        run_core_performance: true,
        run_sufficiency_v2: true,
        max_rows: 10,
        max_symbols: 4,
        max_bytes: 1_000_000,
        require_core_check: false,
        reason_codes: vec![],
    }
}
