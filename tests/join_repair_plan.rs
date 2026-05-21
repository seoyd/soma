#[path = "support/sprint44_support.rs"]
mod sprint44_support;

use soma_zero::{
    JoinRepairActionKind, JoinRepairPlanStatus, MatchKeyNormalizationOptions,
    RowCandleCandidateOptions, build_join_repair_plan, build_match_key_normalization_aggregate,
    build_row_candle_candidate_report, load_symbol_alias_map, load_timeframe_alias_map,
    load_timestamp_policy_map,
};

#[test]
fn join_repair_plan_suggests_explicit_safe_and_operator_repairs() {
    let config =
        sprint44_support::load_audit_config("examples/soma_candle_join_audit_symbol_mismatch.toml");
    let row = sprint44_support::load_row("examples/sprint44_data/repairable_official_bundle.json");
    let pack = sprint44_support::load_pack("examples/soma_candle_pack_official_controlled.toml");
    let normalization = build_match_key_normalization_aggregate(
        &[row.clone()],
        &MatchKeyNormalizationOptions {
            allow_explicit_symbol_alias: false,
            allow_explicit_timeframe_alias: false,
            allow_explicit_timestamp_policy_map: false,
        },
        Some(
            &load_symbol_alias_map("examples/sprint44_data/symbol_alias_map.toml")
                .expect("symbol map"),
        ),
        Some(
            &load_timeframe_alias_map("examples/sprint44_data/timeframe_alias_map.toml")
                .expect("timeframe map"),
        ),
        Some(
            &load_timestamp_policy_map("examples/sprint44_data/timestamp_policy_map.toml")
                .expect("timestamp map"),
        ),
    );
    let candidate_report = build_row_candle_candidate_report(
        &[row],
        &pack,
        &normalization,
        &RowCandleCandidateOptions::default(),
    );
    let plan = build_join_repair_plan(&config, &candidate_report);
    assert_eq!(plan.plan_status, JoinRepairPlanStatus::RepairAvailable);
    assert!(plan.actions.iter().any(|action| action.action_kind
        == JoinRepairActionKind::AddSymbolAlias
        && action.safe_to_apply_automatically));

    let missing_future = sprint44_support::load_audit_config(
        "examples/soma_candle_join_audit_missing_future_window.toml",
    );
    let missing_future_report = soma_zero::OfficialCandleJoinAuditRunner::default()
        .run(&missing_future)
        .expect("audit");
    let future_plan =
        build_join_repair_plan(&missing_future, &missing_future_report.candidate_report);
    assert!(future_plan.actions.iter().any(|action| action.action_kind
        == JoinRepairActionKind::ProvideLongerCandleWindow
        && action.requires_operator_review));
}
