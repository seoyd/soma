#[path = "support/candle_expansion_support.rs"]
mod candle_expansion_support;
mod common;

use soma_zero::{
    ComparableEvidenceSourceClass, OfficialCandleCoverageGapMap, OfficialCandleExpansionPlanConfig,
    OfficialCandleExpansionRunner, OfficialCandleGapConfig, build_official_candle_acquisition_plan,
};

#[test]
fn candle_expansion_gap_plan_runner_and_bundle_are_deterministic() {
    candle_expansion_support::clear_env();
    let timestamps = [
        1_700_000_000_000_u64,
        1_700_086_400_000,
        1_700_172_800_000,
        1_700_259_200_000,
        1_700_345_600_000,
    ];
    let bundle = candle_expansion_support::row_bundle_path(
        "determinism-expansion",
        "AAPL",
        "1d",
        timestamps[0],
        ComparableEvidenceSourceClass::OfficialNonCrypto,
        false,
    );
    let gap_cfg_path = candle_expansion_support::gap_config_path(
        "determinism-expansion",
        vec![bundle.display().to_string()],
        Vec::new(),
    );
    let (csv, provenance, preflight, manifest) = candle_expansion_support::official_csv_fixture(
        "determinism-expansion",
        "aapl_1d",
        "AAPL",
        "1d",
        &timestamps,
        true,
        true,
        true,
    );
    let gap_map_path = candle_expansion_support::manual_gap_map_path(
        "determinism-expansion",
        soma_zero::ProviderMarket::USEquity,
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
    let first_gap = OfficialCandleCoverageGapMap::build(
        &OfficialCandleGapConfig::from_toml_path(&gap_cfg_path).expect("cfg"),
    )
    .expect("first gap");
    let second_gap = OfficialCandleCoverageGapMap::build(
        &OfficialCandleGapConfig::from_toml_path(&gap_cfg_path).expect("cfg"),
    )
    .expect("second gap");
    assert_eq!(first_gap.cells, second_gap.cells);

    let first_plan = build_official_candle_acquisition_plan(&OfficialCandleExpansionPlanConfig {
        plan_id: "determinism-expansion-plan-1".to_string(),
        gap_map_path: Some(gap_map_path.display().to_string()),
        output_root: common::output_dir("determinism-expansion-plan-out-1")
            .display()
            .to_string(),
        ..OfficialCandleExpansionPlanConfig::default()
    })
    .expect("first plan");
    let second_plan = build_official_candle_acquisition_plan(&OfficialCandleExpansionPlanConfig {
        plan_id: "determinism-expansion-plan-2".to_string(),
        gap_map_path: Some(gap_map_path.display().to_string()),
        output_root: common::output_dir("determinism-expansion-plan-out-2")
            .display()
            .to_string(),
        ..OfficialCandleExpansionPlanConfig::default()
    })
    .expect("second plan");
    assert_eq!(first_plan.jobs[0].job_kind, second_plan.jobs[0].job_kind);
    assert_eq!(first_plan.operator_actions, second_plan.operator_actions);

    let first_bundle = OfficialCandleExpansionRunner::default()
        .run_bundle(&OfficialCandleExpansionPlanConfig {
            plan_id: "determinism-expansion-run-1".to_string(),
            gap_config_path: Some(gap_cfg_path.display().to_string()),
            gap_map_path: Some(gap_map_path.display().to_string()),
            output_root: common::output_dir("determinism-expansion-run-out-1")
                .display()
                .to_string(),
            ..OfficialCandleExpansionPlanConfig::default()
        })
        .expect("first bundle");
    let second_bundle = OfficialCandleExpansionRunner::default()
        .run_bundle(&OfficialCandleExpansionPlanConfig {
            plan_id: "determinism-expansion-run-2".to_string(),
            gap_config_path: Some(gap_cfg_path.display().to_string()),
            gap_map_path: Some(gap_map_path.display().to_string()),
            output_root: common::output_dir("determinism-expansion-run-out-2")
                .display()
                .to_string(),
            ..OfficialCandleExpansionPlanConfig::default()
        })
        .expect("second bundle");
    assert_eq!(
        first_bundle.expansion_report.after_counts,
        second_bundle.expansion_report.after_counts
    );
    assert_eq!(
        first_bundle.closure_report.closure_status,
        second_bundle.closure_report.closure_status
    );
}
