#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use soma_zero::{
    BalancedOutcomeCoverageConfig, BarrierProfileRegistryConfig,
    BatchCounterfactualCompletionReport, BatchOutcomeLinkageV3Report,
    DiversityAwareSufficiencyV2Config, MultiRowOfficialEvidenceSet,
    OfficialDiversityRowSelectorConfig, OfficialEvidenceDiversityGapConfig,
    OfficialEvidenceDiversitySweepConfig, OutcomeDiversityAuditConfig,
};

pub fn repo_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

pub fn output_dir(name: &str) -> PathBuf {
    let path = repo_path("target").join("sprint48-tests").join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint48 output dir");
    path
}

pub fn load_json<T: DeserializeOwned>(relative: &str) -> T {
    let text = fs::read_to_string(repo_path(relative)).expect("read json fixture");
    serde_json::from_str(&text).expect("parse json fixture")
}

pub fn all_tp_set() -> MultiRowOfficialEvidenceSet {
    load_json("examples/sprint48_data/diversity_multi_row_set_all_take_profit.json")
}

pub fn mixed_set() -> MultiRowOfficialEvidenceSet {
    load_json("examples/sprint48_data/diversity_multi_row_set_mixed.json")
}

pub fn single_set() -> MultiRowOfficialEvidenceSet {
    load_json("examples/sprint48_data/diversity_multi_row_set_single.json")
}

pub fn crypto_set() -> MultiRowOfficialEvidenceSet {
    load_json("examples/sprint48_data/diversity_multi_row_set_crypto.json")
}

pub fn diagnostic_set() -> MultiRowOfficialEvidenceSet {
    load_json("examples/sprint48_data/diversity_multi_row_set_diagnostics.json")
}

pub fn all_tp_outcomes() -> BatchOutcomeLinkageV3Report {
    load_json("examples/sprint48_data/diversity_all_take_profit_outcomes.json")
}

pub fn mixed_outcomes() -> BatchOutcomeLinkageV3Report {
    load_json("examples/sprint48_data/diversity_mixed_outcomes.json")
}

pub fn single_outcomes() -> BatchOutcomeLinkageV3Report {
    load_json("examples/sprint48_data/diversity_single_outcome.json")
}

pub fn all_tp_counterfactuals() -> BatchCounterfactualCompletionReport {
    load_json("examples/sprint48_data/diversity_all_take_profit_counterfactuals.json")
}

pub fn mixed_counterfactuals() -> BatchCounterfactualCompletionReport {
    load_json("examples/sprint48_data/diversity_mixed_counterfactuals.json")
}

pub fn single_counterfactuals() -> BatchCounterfactualCompletionReport {
    load_json("examples/sprint48_data/diversity_single_counterfactuals.json")
}

pub fn barrier_profiles_primary(name: &str) -> BarrierProfileRegistryConfig {
    let mut config = BarrierProfileRegistryConfig::from_toml_path(&repo_path(
        "examples/soma_barrier_profiles_primary.toml",
    ))
    .expect("primary barrier config");
    config.output_root = output_dir(name).display().to_string();
    config
}

pub fn barrier_profiles_diagnostic(name: &str) -> BarrierProfileRegistryConfig {
    let mut config = BarrierProfileRegistryConfig::from_toml_path(&repo_path(
        "examples/soma_barrier_profiles_diagnostic.toml",
    ))
    .expect("diagnostic barrier config");
    config.output_root = output_dir(name).display().to_string();
    config
}

pub fn diversity_gap_config(name: &str) -> OfficialEvidenceDiversityGapConfig {
    let mut config = OfficialEvidenceDiversityGapConfig::from_toml_path(&repo_path(
        "examples/soma_official_diversity_gap_map_multi_row.toml",
    ))
    .expect("diversity gap config");
    config.output_root = output_dir(name).display().to_string();
    config
}

pub fn row_selector_config(name: &str) -> OfficialDiversityRowSelectorConfig {
    let mut config = OfficialDiversityRowSelectorConfig::from_toml_path(&repo_path(
        "examples/soma_official_diversity_row_select_multi_row.toml",
    ))
    .expect("row selector config");
    let _ = output_dir(name);
    config.sweep_config_path = Some(
        repo_path("examples/sprint48_data/diversity_selector_sweep.toml")
            .display()
            .to_string(),
    );
    config
}

pub fn outcome_audit_config(name: &str) -> OutcomeDiversityAuditConfig {
    let mut config = OutcomeDiversityAuditConfig::from_toml_path(&repo_path(
        "examples/soma_outcome_diversity_audit_multi_row.toml",
    ))
    .expect("outcome audit config");
    config.output_root = output_dir(name).display().to_string();
    config
}

pub fn balanced_coverage_config(name: &str) -> BalancedOutcomeCoverageConfig {
    let mut config = BalancedOutcomeCoverageConfig::from_toml_path(&repo_path(
        "examples/soma_balanced_outcome_coverage_multi_row.toml",
    ))
    .expect("balanced coverage config");
    config.output_root = output_dir(name).display().to_string();
    config
}

pub fn sufficiency_config(name: &str) -> DiversityAwareSufficiencyV2Config {
    let mut config = DiversityAwareSufficiencyV2Config::from_toml_path(&repo_path(
        "examples/soma_diversity_sufficiency_v2_multi_row.toml",
    ))
    .expect("diversity sufficiency config");
    config.output_root = output_dir(name).display().to_string();
    config
}

pub fn sweep_multi_config(name: &str) -> OfficialEvidenceDiversitySweepConfig {
    let mut config = sweep_multi_summary_config(name);
    config.run_committee_official_benchmark = false;
    config.run_outcome_coverage = false;
    config.run_counterfactual_depth_close = false;
    config.run_core_performance = false;
    config
}

pub fn sweep_single_config(name: &str) -> OfficialEvidenceDiversitySweepConfig {
    let mut config = OfficialEvidenceDiversitySweepConfig::from_toml_path(&repo_path(
        "examples/soma_official_evidence_diversity_sweep_single_row.toml",
    ))
    .expect("single sweep config");
    config.output_root = output_dir(name).display().to_string();
    config
}

pub fn sweep_crypto_config(name: &str) -> OfficialEvidenceDiversitySweepConfig {
    let mut config = OfficialEvidenceDiversitySweepConfig::from_toml_path(&repo_path(
        "examples/soma_official_evidence_diversity_sweep_crypto_only.toml",
    ))
    .expect("crypto sweep config");
    config.output_root = output_dir(name).display().to_string();
    config
}

pub fn sweep_diagnostic_config(name: &str) -> OfficialEvidenceDiversitySweepConfig {
    let mut config = OfficialEvidenceDiversitySweepConfig::from_toml_path(&repo_path(
        "examples/soma_official_evidence_diversity_sweep_diagnostics_only.toml",
    ))
    .expect("diagnostic sweep config");
    config.output_root = output_dir(name).display().to_string();
    config
}

pub fn write_file(root: &Path, name: &str, contents: &str) -> PathBuf {
    let path = root.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }
    fs::write(&path, contents).expect("write file");
    path
}

fn toml_string_literal(value: impl AsRef<str>) -> String {
    format!(
        "\"{}\"",
        value.as_ref().replace('\\', "\\\\").replace('\"', "\\\"")
    )
}

fn replace_toml_assignment(contents: &str, key: &str, replacement: &str) -> String {
    let prefix = format!("{key} =");
    contents
        .lines()
        .map(|line| {
            if line.trim_start().starts_with(&prefix) {
                replacement.to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn clone_toml_with_output_root(
    source_relative: &str,
    root: &Path,
    dest_name: &str,
    output_root: &Path,
) -> PathBuf {
    let contents = fs::read_to_string(repo_path(source_relative)).expect("read source toml");
    let updated = replace_toml_assignment(
        &contents,
        "output_root",
        &format!(
            "output_root = {}",
            toml_string_literal(output_root.display().to_string())
        ),
    );
    write_file(root, dest_name, &updated)
}

fn clone_toml_with_replacements(
    source_relative: &str,
    root: &Path,
    dest_name: &str,
    replacements: &[(&str, String)],
) -> PathBuf {
    let mut contents = fs::read_to_string(repo_path(source_relative)).expect("read source toml");
    for (key, replacement) in replacements {
        contents = replace_toml_assignment(&contents, key, replacement);
    }
    write_file(root, dest_name, &contents)
}

pub fn sweep_multi_summary_config(name: &str) -> OfficialEvidenceDiversitySweepConfig {
    let root = output_dir(name);
    let plan_path = clone_toml_with_output_root(
        "examples/sprint48_data/diversity_sweep_current.toml",
        &root,
        "diversity_sweep_current.toml",
        &root.join("sweep_plan_outputs"),
    );
    let row_selector_path = clone_toml_with_replacements(
        "examples/soma_official_diversity_row_select_multi_row.toml",
        &root,
        "official_diversity_row_select_multi_row.toml",
        &[(
            "sweep_config_path",
            format!(
                "sweep_config_path = {}",
                toml_string_literal(plan_path.display().to_string())
            ),
        )],
    );
    let gap_map_path = clone_toml_with_output_root(
        "examples/soma_official_diversity_gap_map_multi_row.toml",
        &root,
        "official_diversity_gap_map_multi_row.toml",
        &root.join("gap_map_outputs"),
    );
    let committee_benchmark_path = clone_toml_with_output_root(
        "examples/soma_committee_official_benchmark_controlled.toml",
        &root,
        "committee_official_benchmark_controlled.toml",
        &root.join("committee_benchmark_outputs"),
    );
    let outcome_coverage_path = clone_toml_with_replacements(
        "examples/soma_committee_outcome_coverage_controlled.toml",
        &root,
        "committee_outcome_coverage_controlled.toml",
        &[
            (
                "official_benchmark_report_paths",
                format!(
                    "official_benchmark_report_paths = [{}]",
                    toml_string_literal(committee_benchmark_path.display().to_string())
                ),
            ),
            (
                "output_root",
                format!(
                    "output_root = {}",
                    toml_string_literal(
                        root.join("committee_outcome_coverage_outputs")
                            .display()
                            .to_string()
                    )
                ),
            ),
        ],
    );
    let core_performance_path = clone_toml_with_replacements(
        "examples/soma_core_performance_controlled.toml",
        &root,
        "core_performance_controlled.toml",
        &[
            (
                "committee_outcome_coverage_paths",
                format!(
                    "committee_outcome_coverage_paths = [{}]",
                    toml_string_literal(outcome_coverage_path.display().to_string())
                ),
            ),
            (
                "output_root",
                format!(
                    "output_root = {}",
                    toml_string_literal(
                        root.join("core_performance_outputs").display().to_string()
                    )
                ),
            ),
        ],
    );
    let counterfactual_depth_path = clone_toml_with_replacements(
        "examples/soma_counterfactual_depth_close_controlled.toml",
        &root,
        "counterfactual_depth_close_controlled.toml",
        &[
            (
                "core_performance_config_path",
                format!(
                    "core_performance_config_path = {}",
                    toml_string_literal(core_performance_path.display().to_string())
                ),
            ),
            (
                "output_root",
                format!(
                    "output_root = {}",
                    toml_string_literal(
                        root.join("counterfactual_depth_outputs")
                            .display()
                            .to_string()
                    )
                ),
            ),
        ],
    );

    let mut config = OfficialEvidenceDiversitySweepConfig::from_toml_path(&repo_path(
        "examples/soma_official_evidence_diversity_sweep_multi_row.toml",
    ))
    .expect("multi sweep config");
    config.output_root = root.display().to_string();
    config.sweep_config_path = Some(plan_path.display().to_string());
    config.row_selector_config_path = Some(row_selector_path.display().to_string());
    config.diversity_gap_map_path = Some(gap_map_path.display().to_string());
    config.committee_official_benchmark_config_path =
        Some(committee_benchmark_path.display().to_string());
    config.outcome_coverage_config_path = Some(outcome_coverage_path.display().to_string());
    config.counterfactual_depth_closure_config_path =
        Some(counterfactual_depth_path.display().to_string());
    config.core_performance_config_path = Some(core_performance_path.display().to_string());
    config
}
