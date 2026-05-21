#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use soma_zero::{
    CompleteRowClosureV2Config, CounterfactualCompletionV2Config, FutureWindowRequirementConfig,
    OfficialFutureWindowExtensionConfig, OutcomeLinkageV3Config,
};

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn example_path(path: &str) -> PathBuf {
    repo_root().join(path)
}

pub fn output_dir(name: &str) -> PathBuf {
    let path = repo_root().join("target").join("sprint46-tests").join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint46 output dir");
    path
}

pub fn future_window_short_config(name: &str) -> FutureWindowRequirementConfig {
    FutureWindowRequirementConfig {
        requirement_id: name.to_string(),
        official_ready_inventory_paths: vec![
            example_path("examples/sprint46_data/official_ready_row_short_window.json")
                .display()
                .to_string(),
        ],
        candle_coverage_pack_paths: vec![
            example_path("examples/sprint46_data/aapl_1d_extended.csv")
                .display()
                .to_string(),
        ],
        output_root: output_dir(name).display().to_string(),
        default_horizon_bars: 3,
        max_rows: 10,
        max_symbols: 1,
        max_bytes: 1_000_000,
        ..FutureWindowRequirementConfig::default()
    }
}

pub fn future_window_extended_config(name: &str) -> FutureWindowRequirementConfig {
    FutureWindowRequirementConfig {
        requirement_id: name.to_string(),
        official_ready_inventory_paths: vec![
            example_path("examples/sprint46_data/official_ready_row_extended_window.json")
                .display()
                .to_string(),
        ],
        candle_coverage_pack_paths: vec![
            example_path("examples/sprint46_data/aapl_1d_extended.csv")
                .display()
                .to_string(),
        ],
        output_root: output_dir(name).display().to_string(),
        default_horizon_bars: 3,
        max_rows: 10,
        max_symbols: 1,
        max_bytes: 1_000_000,
        ..FutureWindowRequirementConfig::default()
    }
}

pub fn extension_config(name: &str) -> OfficialFutureWindowExtensionConfig {
    OfficialFutureWindowExtensionConfig {
        extension_id: name.to_string(),
        future_window_requirement_path: Some(
            example_path("examples/soma_future_window_requirements_official_replication.toml")
                .display()
                .to_string(),
        ),
        official_canonical_csv_paths: vec![
            example_path("examples/sprint46_data/aapl_1d_extended.csv")
                .display()
                .to_string(),
        ],
        provenance_paths: vec![
            example_path("examples/sprint46_data/aapl_1d_extended_provenance.json")
                .display()
                .to_string(),
        ],
        preflight_report_paths: vec![
            example_path("examples/sprint46_data/aapl_1d_extended_preflight.json")
                .display()
                .to_string(),
        ],
        output_root: output_dir(name).display().to_string(),
        max_jobs: 5,
        max_symbols: 1,
        max_rows_per_job: 10,
        max_requests_per_job: 2,
        max_total_bytes: 1_000_000,
        prefer_local_extension: true,
        generate_provider_jobs: false,
        run_local_extension_jobs: true,
        run_provider_collection_jobs: false,
        ..OfficialFutureWindowExtensionConfig::default()
    }
}

pub fn outcome_config(name: &str) -> OutcomeLinkageV3Config {
    OutcomeLinkageV3Config {
        linkage_id: name.to_string(),
        official_ready_inventory_path: Some(
            example_path("examples/sprint46_data/official_ready_row_extended_window.json")
                .display()
                .to_string(),
        ),
        extended_candle_pack_paths: vec![
            example_path("examples/sprint46_data/aapl_1d_extended.csv")
                .display()
                .to_string(),
        ],
        output_root: output_dir(name).display().to_string(),
        default_horizon_bars: 3,
        take_profit_pct: 0.02,
        stop_loss_pct: 0.01,
        cost_bps: 5.0,
        slippage_bps: 2.0,
        ..OutcomeLinkageV3Config::default()
    }
}

pub fn counterfactual_config(name: &str) -> CounterfactualCompletionV2Config {
    CounterfactualCompletionV2Config {
        completion_id: name.to_string(),
        outcome_linkage_v3_path: Some(
            example_path("examples/soma_outcome_linkage_v3_official_replication.toml")
                .display()
                .to_string(),
        ),
        complete_row_closure_path: Some(
            example_path("examples/sprint46_data/comparable_official_extended_window_bundle.json")
                .display()
                .to_string(),
        ),
        output_root: output_dir(name).display().to_string(),
        ..CounterfactualCompletionV2Config::default()
    }
}

pub fn closure_v2_config(name: &str) -> CompleteRowClosureV2Config {
    CompleteRowClosureV2Config {
        closure_id: name.to_string(),
        complete_row_closure_config_path: Some(
            example_path("examples/sprint46_data/comparable_official_extended_window_bundle.json")
                .display()
                .to_string(),
        ),
        outcome_linkage_v3_config_path: Some(
            example_path("examples/soma_outcome_linkage_v3_official_replication.toml")
                .display()
                .to_string(),
        ),
        counterfactual_completion_v2_config_path: Some(
            example_path("examples/soma_counterfactual_complete_v2_official_replication.toml")
                .display()
                .to_string(),
        ),
        output_root: output_dir(name).display().to_string(),
        run_core_scorecard_rerun: true,
        ..CompleteRowClosureV2Config::default()
    }
}
