#[path = "support/candle_expansion_support.rs"]
mod candle_expansion_support;
mod common;

use soma_zero::{
    ComparableEvidenceSourceClass, OfficialCandleExpansionFinalStatus,
    OfficialCandleExpansionPlanConfig, OfficialCandleExpansionRunner, OfficialCandleGapStatus,
    ProviderMarket,
};

#[test]
fn candle_expansion_handles_no_gaps_missing_auth_missing_csv_and_local_import() {
    candle_expansion_support::clear_env();
    let timestamps = [
        1_700_000_000_000_u64,
        1_700_086_400_000,
        1_700_172_800_000,
        1_700_259_200_000,
        1_700_345_600_000,
    ];
    let bundle = candle_expansion_support::row_bundle_path(
        "candle-expansion-suite",
        "AAPL",
        "1d",
        timestamps[0],
        ComparableEvidenceSourceClass::OfficialNonCrypto,
        false,
    );
    let (csv, provenance, preflight, manifest) = candle_expansion_support::official_csv_fixture(
        "candle-expansion-suite",
        "aapl_1d",
        "AAPL",
        "1d",
        &timestamps,
        true,
        true,
        true,
    );
    let pack_cfg = candle_expansion_support::candle_coverage_support::pack_config(
        "candle-expansion-suite-pack",
        vec![csv.display().to_string()],
        provenance.clone(),
        preflight.clone(),
        manifest.clone(),
    );
    let pack_path = candle_expansion_support::candle_coverage_support::write_pack_config_file(
        "candle-expansion-suite-pack",
        &pack_cfg,
    );
    let gap_cfg_path = candle_expansion_support::gap_config_path(
        "candle-expansion-suite",
        vec![bundle.display().to_string()],
        vec![pack_path.display().to_string()],
    );
    let report = OfficialCandleExpansionRunner::default()
        .run(&OfficialCandleExpansionPlanConfig {
            plan_id: "candle-expansion-suite-plan".to_string(),
            gap_config_path: Some(gap_cfg_path.display().to_string()),
            output_root: common::output_dir("candle-expansion-suite-out")
                .display()
                .to_string(),
            ..OfficialCandleExpansionPlanConfig::default()
        })
        .expect("run");
    assert_eq!(
        report.gap_map.gap_status,
        OfficialCandleGapStatus::NoGapsDetected
    );

    let auth_gap_cfg = candle_expansion_support::gap_config_path(
        "candle-expansion-suite-missing-auth",
        vec![bundle.display().to_string()],
        Vec::new(),
    );
    let missing_auth = OfficialCandleExpansionRunner::default()
        .run(&OfficialCandleExpansionPlanConfig {
            plan_id: "candle-expansion-suite-missing-auth-plan".to_string(),
            gap_config_path: Some(auth_gap_cfg.display().to_string()),
            allow_local_import: false,
            output_root: common::output_dir("candle-expansion-suite-missing-auth-out")
                .display()
                .to_string(),
            ..OfficialCandleExpansionPlanConfig::default()
        })
        .expect("missing auth");
    assert_eq!(
        missing_auth.final_status,
        OfficialCandleExpansionFinalStatus::MissingAuth
    );
}

#[test]
fn candle_expansion_preserves_source_boundaries_and_determinism() {
    candle_expansion_support::clear_env();
    let yfinance_bundle = candle_expansion_support::row_bundle_path(
        "candle-expansion-suite-yfinance",
        "AAPL",
        "1d",
        1_700_000_000_000,
        ComparableEvidenceSourceClass::YFinanceResearch,
        false,
    );
    let yfinance_cfg = candle_expansion_support::gap_config_path(
        "candle-expansion-suite-yfinance",
        vec![yfinance_bundle.display().to_string()],
        Vec::new(),
    );
    let yfinance_map = candle_expansion_support::manual_gap_map_path(
        "candle-expansion-suite-yfinance",
        ProviderMarket::USEquity,
        "AAPL",
        "1d",
        ComparableEvidenceSourceClass::YFinanceResearch,
        Vec::new(),
    );
    let first = OfficialCandleExpansionRunner::default()
        .run(&OfficialCandleExpansionPlanConfig {
            plan_id: "candle-expansion-suite-yf-1".to_string(),
            gap_config_path: Some(yfinance_cfg.display().to_string()),
            gap_map_path: Some(yfinance_map.display().to_string()),
            output_root: common::output_dir("candle-expansion-suite-yf-1")
                .display()
                .to_string(),
            ..OfficialCandleExpansionPlanConfig::default()
        })
        .expect("first");
    let second = OfficialCandleExpansionRunner::default()
        .run(&OfficialCandleExpansionPlanConfig {
            plan_id: "candle-expansion-suite-yf-2".to_string(),
            gap_config_path: Some(yfinance_cfg.display().to_string()),
            gap_map_path: Some(yfinance_map.display().to_string()),
            output_root: common::output_dir("candle-expansion-suite-yf-2")
                .display()
                .to_string(),
            ..OfficialCandleExpansionPlanConfig::default()
        })
        .expect("second");
    assert_eq!(
        first.final_status,
        OfficialCandleExpansionFinalStatus::DiagnosticCandleCoverageOnly
    );
    assert_eq!(first.after_counts.gap_count, second.after_counts.gap_count);
}
