#[path = "support/candle_expansion_support.rs"]
mod candle_expansion_support;
mod common;

use soma_zero::{OfficialCandleExpansionPlanConfig, build_official_candle_acquisition_plan};

#[test]
fn expansion_plan_config_defaults_validate_and_reject_remote_paths() {
    let config = OfficialCandleExpansionPlanConfig::default();
    assert!(!config.run_collection_jobs);
    assert!(config.run_import_jobs);
    assert!(config.validate().is_ok());

    let remote = OfficialCandleExpansionPlanConfig {
        gap_map_path: Some("https://example.com/gap.json".to_string()),
        ..OfficialCandleExpansionPlanConfig::default()
    };
    assert!(remote.validate().unwrap_err().contains("local"));
}

#[test]
fn expansion_plan_config_enforces_bounds() {
    let mut config = OfficialCandleExpansionPlanConfig::default();
    config.max_jobs = 11;
    assert!(config.validate().unwrap_err().contains("max_jobs"));
    config.max_jobs = 10;
    config.max_rows_per_job = 501;
    assert!(config.validate().unwrap_err().contains("max_rows_per_job"));
    config.max_rows_per_job = 500;
    config.max_requests_per_job = 11;
    assert!(
        config
            .validate()
            .unwrap_err()
            .contains("max_requests_per_job")
    );
    config.max_requests_per_job = 10;
    config.max_total_bytes = 5_000_001;
    assert!(config.validate().unwrap_err().contains("max_total_bytes"));
}

#[test]
fn expansion_plan_builds_missing_auth_actions_and_is_deterministic() {
    candle_expansion_support::clear_env();
    let gap_map = candle_expansion_support::manual_gap_map_path(
        "plan-missing-auth",
        soma_zero::ProviderMarket::USEquity,
        "AAPL",
        "1d",
        soma_zero::ComparableEvidenceSourceClass::OfficialNonCrypto,
        Vec::new(),
    );
    let first = build_official_candle_acquisition_plan(&OfficialCandleExpansionPlanConfig {
        plan_id: "plan-missing-auth-1".to_string(),
        gap_map_path: Some(gap_map.display().to_string()),
        allow_local_import: false,
        output_root: common::output_dir("plan-missing-auth-1")
            .display()
            .to_string(),
        ..OfficialCandleExpansionPlanConfig::default()
    })
    .expect("first plan");
    let second = build_official_candle_acquisition_plan(&OfficialCandleExpansionPlanConfig {
        plan_id: "plan-missing-auth-2".to_string(),
        gap_map_path: Some(gap_map.display().to_string()),
        allow_local_import: false,
        output_root: common::output_dir("plan-missing-auth-2")
            .display()
            .to_string(),
        ..OfficialCandleExpansionPlanConfig::default()
    })
    .expect("second plan");
    assert!(
        first
            .operator_actions
            .iter()
            .any(|action| action.action_id == "set-alphavantage-api-key")
    );
    assert_eq!(first.jobs[0].job_kind, second.jobs[0].job_kind);
    assert_eq!(
        first.operator_actions[0].action_id,
        second.operator_actions[0].action_id
    );
}
