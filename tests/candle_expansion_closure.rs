#[path = "support/candle_expansion_support.rs"]
mod candle_expansion_support;
mod common;

use soma_zero::{
    CandleExpansionClosureStatus, ComparableEvidenceSourceClass, OfficialCandleExpansionPlanConfig,
    OfficialCandleExpansionRunner, ProviderMarket,
};

#[test]
fn closure_report_counts_official_series_and_detects_improvement_and_blocks() {
    candle_expansion_support::clear_env();
    let timestamps = [
        1_700_000_000_000_u64,
        1_700_086_400_000,
        1_700_172_800_000,
        1_700_259_200_000,
        1_700_345_600_000,
    ];
    let bundle = candle_expansion_support::row_bundle_path(
        "closure-success",
        "AAPL",
        "1d",
        timestamps[0],
        ComparableEvidenceSourceClass::OfficialNonCrypto,
        false,
    );
    let gap_cfg = candle_expansion_support::gap_config_path(
        "closure-success",
        vec![bundle.display().to_string()],
        Vec::new(),
    );
    let (csv, provenance, preflight, manifest) = candle_expansion_support::official_csv_fixture(
        "closure-success",
        "aapl_1d",
        "AAPL",
        "1d",
        &timestamps,
        true,
        true,
        true,
    );
    let gap_map = candle_expansion_support::manual_gap_map_path(
        "closure-success",
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
    let bundle_report = OfficialCandleExpansionRunner::default()
        .run_bundle(&OfficialCandleExpansionPlanConfig {
            plan_id: "closure-success-plan".to_string(),
            gap_config_path: Some(gap_cfg.display().to_string()),
            gap_map_path: Some(gap_map.display().to_string()),
            output_root: common::output_dir("closure-success-plan-out")
                .display()
                .to_string(),
            ..OfficialCandleExpansionPlanConfig::default()
        })
        .expect("bundle report");
    assert!(bundle_report.closure_report.added_official_series >= 1);
    assert!(
        bundle_report
            .closure_report
            .added_non_crypto_official_series
            >= 1
    );
    assert!(bundle_report.closure_report.added_backfilled_rows >= 1);
    assert!(matches!(
        bundle_report.closure_report.closure_status,
        CandleExpansionClosureStatus::GapClosed
            | CandleExpansionClosureStatus::GapImproved
            | CandleExpansionClosureStatus::GapMovedToOfficialEvidence
    ));

    let missing_auth_bundle = OfficialCandleExpansionRunner::default()
        .run_bundle(&OfficialCandleExpansionPlanConfig {
            plan_id: "closure-auth-plan".to_string(),
            gap_config_path: Some(gap_cfg.display().to_string()),
            allow_local_import: false,
            output_root: common::output_dir("closure-auth-plan-out")
                .display()
                .to_string(),
            ..OfficialCandleExpansionPlanConfig::default()
        })
        .expect("missing auth bundle");
    assert_eq!(
        missing_auth_bundle.closure_report.closure_status,
        CandleExpansionClosureStatus::BlockedByAuth
    );
}
