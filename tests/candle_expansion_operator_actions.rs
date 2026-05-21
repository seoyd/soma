#[path = "support/candle_expansion_support.rs"]
mod candle_expansion_support;
mod common;

use soma_zero::{
    ComparableEvidenceSourceClass, OfficialCandleCoverageGapMap, OfficialCandleExpansionPlanConfig,
    ProviderMarket, build_candle_expansion_operator_actions,
    build_official_candle_acquisition_plan,
};

#[test]
fn operator_actions_cover_missing_auth_approval_endpoint_data_and_are_safe() {
    candle_expansion_support::clear_env();
    let gap_map_path = candle_expansion_support::manual_gap_map_path(
        "actions-gap",
        ProviderMarket::KoreanEquity,
        "005930",
        "1d",
        ComparableEvidenceSourceClass::OfficialNonCrypto,
        Vec::new(),
    );
    let plan = build_official_candle_acquisition_plan(&OfficialCandleExpansionPlanConfig {
        plan_id: "actions-plan".to_string(),
        gap_map_path: Some(gap_map_path.display().to_string()),
        allow_local_import: false,
        output_root: common::output_dir("actions-plan-out").display().to_string(),
        ..OfficialCandleExpansionPlanConfig::default()
    })
    .expect("plan");
    let actions = build_candle_expansion_operator_actions(
        &OfficialCandleCoverageGapMap::from_json_path(&gap_map_path).expect("map"),
        &plan.jobs,
    );
    let ids = actions
        .iter()
        .map(|action| action.action_id.as_str())
        .collect::<Vec<_>>();
    assert!(ids.contains(&"wait-for-krx-approval"));
    assert!(ids.contains(&"provide-official-canonical-csv"));
    assert!(ids.contains(&"run-candle-coverage-close"));
    for action in &actions {
        let text = action.to_text();
        assert!(!text.contains("secret"));
        assert!(!text.contains("broker"));
        assert!(!text.contains("order"));
        assert!(!text.contains("account"));
    }
}

#[test]
fn operator_actions_include_provenance_and_preflight_guidance_and_are_deterministic() {
    let timestamps = [1_700_000_000_000_u64, 1_700_086_400_000, 1_700_172_800_000];
    let (csv, _, _, _) = candle_expansion_support::official_csv_fixture(
        "actions-local",
        "aapl_1d",
        "AAPL",
        "1d",
        &timestamps,
        false,
        false,
        false,
    );
    let gap_map_path = candle_expansion_support::manual_gap_map_path(
        "actions-local",
        ProviderMarket::USEquity,
        "AAPL",
        "1d",
        ComparableEvidenceSourceClass::OfficialNonCrypto,
        vec![csv.display().to_string()],
    );
    let map = OfficialCandleCoverageGapMap::from_json_path(&gap_map_path).expect("map");
    let actions_first = build_candle_expansion_operator_actions(&map, &[]);
    let actions_second = build_candle_expansion_operator_actions(&map, &[]);
    assert!(
        actions_first
            .iter()
            .any(|action| action.action_id == "provide-official-canonical-csv")
    );
    assert_eq!(actions_first, actions_second);
}
