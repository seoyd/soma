mod common;

use soma_zero::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass, CounterfactualDepthPlan,
    CounterfactualGapKind, ProviderMarket,
};

fn row(id: &str) -> ComparableCommitteeEvidenceRow {
    ComparableCommitteeEvidenceRow {
        row_id: id.to_string(),
        symbol: "AAPL".to_string(),
        market: ProviderMarket::USEquity,
        timeframe: "1d".to_string(),
        horizon_bars: 24,
        timestamp_ms: 1_700_000_000_000,
        source_kind: "OfficialApiCollected".to_string(),
        source_class: ComparableEvidenceSourceClass::OfficialNonCrypto,
        scenario_row_id: Some(id.to_string()),
        committee_decision_id: None,
        committee_final_action: "Approve".to_string(),
        chair_decision: None,
        risk_governor_decision: None,
        baseline_action: None,
        external_action: None,
        no_trade_baseline_action: "NoTrade".to_string(),
        outcome_label: None,
        net_return_pct: None,
        cost_bps: 0.0,
        slippage_bps: 0.0,
        committee_vs_baseline_delta: None,
        committee_vs_notrade_delta: None,
        risk_denied_value_proxy: None,
        no_trade_value_proxy: None,
        outcome_reference_available: false,
        baseline_reference_available: false,
        no_trade_counterfactual_available: false,
        risk_denied_counterfactual_available: false,
        external_reference_available: false,
        row_level: false,
        summary_derived: true,
        no_lookahead_safe: true,
        official_readiness_eligible: true,
        diagnostic_only: false,
        candle_coverage_available: false,
        matched_candle_series_id: None,
        candle_match_status: None,
        candle_official_ready_match: false,
        candle_benchmark_ready_match: false,
        candle_diagnostic_only: false,
        reason_codes: Vec::new(),
    }
}

#[test]
fn depth_plan_flags_missing_references_and_summary_only_rows() {
    let config = ComparableCommitteeEvidenceConfig {
        comparable_id: "depth-plan".to_string(),
        output_root: common::output_dir("depth-plan").display().to_string(),
        ..ComparableCommitteeEvidenceConfig::default()
    };
    let bundle = ComparableCommitteeEvidenceBundle::from_rows(&config, vec![row("gap-row")]);
    let plan = CounterfactualDepthPlan::from_bundle(&config, &bundle);

    assert_eq!(plan.rows_missing_outcome, 1);
    assert_eq!(plan.rows_missing_baseline, 1);
    assert_eq!(plan.rows_missing_no_trade, 1);
    assert_eq!(plan.rows_missing_risk_denied, 1);
    assert!(
        plan.items
            .iter()
            .any(|item| item.gap_kind == CounterfactualGapKind::SummaryDerivedOnly)
    );
}
