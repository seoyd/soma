mod common;
#[path = "support/sprint46_support.rs"]
mod sprint46_support;

use soma_zero::{
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass,
    CounterfactualCompletionV2RecordStatus, CounterfactualCompletionV2Runner,
    CounterfactualCompletionV2Status, OutcomeLinkageV3Record, OutcomeLinkageV3Report,
    OutcomeLinkageV3Status, ProviderMarket, ReasonCode,
};

#[test]
fn counterfactual_completion_v2_builds_no_trade_and_risk_denied_from_outcomes() {
    let report = CounterfactualCompletionV2Runner::default()
        .run(&sprint46_support::counterfactual_config(
            "counterfactual-complete-v2",
        ))
        .expect("counterfactual completion");
    assert_eq!(report.completed_count, 1);
    assert_eq!(report.no_trade_built_count, 1);
    assert_eq!(report.risk_denied_built_count, 1);
    assert_eq!(
        report.completion_status,
        CounterfactualCompletionV2Status::OfficialCounterfactualsImproved
    );
    assert!(report.records[0].missed_gain_value.is_some());
    assert!(report.records[0].avoided_loss_value.is_none());
}

#[test]
fn counterfactual_completion_v2_requires_outcome_reference() {
    let row = ComparableCommitteeEvidenceRow {
        row_id: "row-a".to_string(),
        symbol: "AAPL".to_string(),
        market: ProviderMarket::USEquity,
        timeframe: "1d".to_string(),
        horizon_bars: 3,
        timestamp_ms: 1700000000000,
        source_kind: "OfficialApiCollected".to_string(),
        source_class: ComparableEvidenceSourceClass::OfficialNonCrypto,
        scenario_row_id: Some("scenario-a".to_string()),
        committee_decision_id: Some("committee-a".to_string()),
        committee_final_action: "Approve".to_string(),
        chair_decision: Some("Approve".to_string()),
        risk_governor_decision: Some("Reject".to_string()),
        baseline_action: Some("Approve".to_string()),
        external_action: None,
        no_trade_baseline_action: "NoTrade".to_string(),
        outcome_label: None,
        net_return_pct: None,
        cost_bps: 5.0,
        slippage_bps: 2.0,
        committee_vs_baseline_delta: None,
        committee_vs_notrade_delta: None,
        risk_denied_value_proxy: None,
        no_trade_value_proxy: None,
        outcome_reference_available: false,
        baseline_reference_available: false,
        no_trade_counterfactual_available: false,
        risk_denied_counterfactual_available: false,
        external_reference_available: false,
        row_level: true,
        summary_derived: false,
        no_lookahead_safe: true,
        official_readiness_eligible: true,
        diagnostic_only: false,
        candle_coverage_available: true,
        matched_candle_series_id: Some("aapl_1d_extended".to_string()),
        candle_match_status: Some("Matched".to_string()),
        candle_official_ready_match: true,
        candle_benchmark_ready_match: true,
        candle_diagnostic_only: false,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    let outcome = OutcomeLinkageV3Report {
        linkage_id: "missing-outcome".to_string(),
        records: vec![OutcomeLinkageV3Record {
            row_id: "row-a".to_string(),
            scenario_row_id: Some("scenario-a".to_string()),
            candle_series_id: Some("aapl_1d_extended".to_string()),
            status: soma_zero::OutcomeLinkageV3RecordStatus::SkippedMissingFutureBars,
            outcome_reference: None,
            net_return_pct: None,
            mfe_pct: None,
            mae_pct: None,
            cost_bps: 5.0,
            slippage_bps: 2.0,
            reason_codes: vec![ReasonCode::InsufficientBars],
        }],
        generated_outcome_count: 0,
        skipped_missing_future_bars: 1,
        skipped_timestamp_mismatch: 0,
        skipped_horizon_mismatch: 0,
        rejected_no_lookahead: 0,
        official_outcome_count: 0,
        diagnostic_outcome_count: 0,
        linkage_status: OutcomeLinkageV3Status::StillNeedFutureBars,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    let report = CounterfactualCompletionV2Runner::default()
        .run_from_inputs(
            &sprint46_support::counterfactual_config("counterfactual-missing-outcome"),
            &outcome,
            &[row],
        )
        .expect("counterfactual missing outcome");
    assert_eq!(
        report.records[0].status,
        CounterfactualCompletionV2RecordStatus::SkippedMissingOutcome
    );
}
