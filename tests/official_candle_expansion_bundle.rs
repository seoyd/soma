#[path = "support/candle_expansion_support.rs"]
mod candle_expansion_support;
mod common;

use soma_zero::{
    ComparableEvidenceSourceClass, OfficialCandleExpansionPlanConfig,
    OfficialCandleExpansionRunner, ProviderMarket, ReasonCode,
};

#[test]
fn expansion_bundle_writes_storage_report_and_counts_artifacts_deterministically() {
    candle_expansion_support::clear_env();
    let timestamps = [
        1_700_000_000_000_u64,
        1_700_086_400_000,
        1_700_172_800_000,
        1_700_259_200_000,
        1_700_345_600_000,
    ];
    let bundle = candle_expansion_support::row_bundle_path(
        "bundle-success",
        "AAPL",
        "1d",
        timestamps[0],
        ComparableEvidenceSourceClass::OfficialNonCrypto,
        false,
    );
    let gap_cfg = candle_expansion_support::gap_config_path(
        "bundle-success",
        vec![bundle.display().to_string()],
        Vec::new(),
    );
    let (csv, provenance, preflight, manifest) = candle_expansion_support::official_csv_fixture(
        "bundle-success",
        "aapl_1d",
        "AAPL",
        "1d",
        &timestamps,
        true,
        true,
        true,
    );
    let gap_map = candle_expansion_support::manual_gap_map_path(
        "bundle-success",
        ProviderMarket::USEquity,
        "AAPL",
        "1d",
        ComparableEvidenceSourceClass::OfficialNonCrypto,
        vec![
            csv.display().to_string(),
            provenance[0].clone(),
            preflight[0].clone(),
            manifest[0].clone(),
        ],
    );
    let first = OfficialCandleExpansionRunner::default()
        .run_bundle(&OfficialCandleExpansionPlanConfig {
            plan_id: "bundle-success-plan-1".to_string(),
            gap_config_path: Some(gap_cfg.display().to_string()),
            gap_map_path: Some(gap_map.display().to_string()),
            output_root: common::output_dir("bundle-success-plan-out-1")
                .display()
                .to_string(),
            ..OfficialCandleExpansionPlanConfig::default()
        })
        .expect("first bundle");
    let second = OfficialCandleExpansionRunner::default()
        .run_bundle(&OfficialCandleExpansionPlanConfig {
            plan_id: "bundle-success-plan-2".to_string(),
            gap_config_path: Some(gap_cfg.display().to_string()),
            gap_map_path: Some(gap_map.display().to_string()),
            output_root: common::output_dir("bundle-success-plan-out-2")
                .display()
                .to_string(),
            ..OfficialCandleExpansionPlanConfig::default()
        })
        .expect("second bundle");
    assert!(first.storage_report.total_bytes > 0);
    assert!(first.storage_report.artifact_count >= 4);
    assert!(first.storage_report.deleted_artifacts.is_empty());
    assert_eq!(
        first.storage_report.largest_artifacts,
        second.storage_report.largest_artifacts
    );

    let exceeded = OfficialCandleExpansionRunner::default()
        .run_bundle(&OfficialCandleExpansionPlanConfig {
            plan_id: "bundle-budget-plan".to_string(),
            gap_config_path: Some(gap_cfg.display().to_string()),
            gap_map_path: Some(gap_map.display().to_string()),
            max_total_bytes: 1,
            output_root: common::output_dir("bundle-budget-plan-out")
                .display()
                .to_string(),
            ..OfficialCandleExpansionPlanConfig::default()
        })
        .expect("budget bundle");
    assert!(exceeded.storage_report.budget_exceeded);
    assert!(
        exceeded
            .storage_report
            .reason_codes
            .contains(&ReasonCode::BudgetExceeded)
    );
}
