use std::collections::BTreeMap;

use soma_zero::{
    CalibrationMetrics, ChairMetrics, DecisionMetrics, FeatureSchema, FoldReport, NoTradeMetrics,
    PersonaFoldMetrics, RegimeMetrics, RiskAiInteractionReport, RiskGovernorMetrics, Timeframe,
    TradeMetrics, WalkForwardAggregateMetrics, WalkForwardConfig, WalkForwardFold,
    WalkForwardReport,
};

fn walk_forward_report(
    denied_by_risk: usize,
    no_trade: usize,
    approve_candidate_count: usize,
    defensive_value: f64,
    max_drawdown_pct: f64,
) -> WalkForwardReport {
    let decision_metrics = DecisionMetrics {
        total_decisions: 10,
        executed: 3,
        denied_by_risk,
        no_trade,
        require_confirm_count: 0,
        approve_candidate_count,
        reason_code_counts: BTreeMap::from([
            ("DeniedByDefault".to_string(), denied_by_risk),
            ("CandidateApproved".to_string(), approve_candidate_count),
        ]),
    };
    let risk_metrics = RiskGovernorMetrics {
        denied_count: denied_by_risk,
        emergency_stop_count: 0,
        cooldown_count: 0,
        avoided_loss_count: 2,
        missed_gain_count: 1,
        defensive_value,
        opportunity_cost: 0.1,
    };
    WalkForwardReport {
        symbol: "BTC-USDT".to_string(),
        timeframe: Timeframe::OneMinute,
        config: WalkForwardConfig::default(),
        folds: vec![FoldReport {
            fold_id: 0,
            fold: WalkForwardFold {
                fold_id: 0,
                train_start_index: 0,
                train_end_index: 4,
                validation_start_index: Some(5),
                validation_end_index: Some(6),
                test_start_index: 7,
                test_end_index: 9,
                embargo_start_index: None,
                embargo_end_index: None,
                reason_codes: vec![],
            },
            train_rows: 5,
            validation_rows: 2,
            test_rows: 3,
            leakage_report: soma_zero::LeakageReport {
                has_leakage: false,
                warnings: vec![],
                unsafe_rows_count: 0,
                checked_rows_count: 10,
                reason_codes: vec![],
            },
            test_trade_metrics: TradeMetrics {
                total_trades: 3,
                wins: 2,
                losses: 1,
                neutrals: 0,
                win_rate: 0.66,
                avg_win_pct: 0.02,
                avg_loss_pct: -0.01,
                gross_return_pct: 0.03,
                net_return_pct: 0.02,
                profit_factor: Some(1.5),
                max_drawdown_pct,
                avg_bars_held: 2.0,
                reason_codes: vec![],
            },
            test_decision_metrics: decision_metrics.clone(),
            test_no_trade_metrics: NoTradeMetrics {
                no_trade_count: no_trade,
                avoided_loss_count: 2,
                missed_gain_count: 1,
                avg_avoided_loss_score: 0.2,
                avg_missed_gain_penalty: -0.1,
                net_silence_value: 0.1,
            },
            test_risk_metrics: risk_metrics.clone(),
            calibration_metrics: CalibrationMetrics {
                brier_score: 0.1,
                calibration_bins: vec![],
                expected_calibration_error: Some(0.03),
            },
            regime_metrics: Vec::<RegimeMetrics>::new(),
            persona_metrics: Vec::<PersonaFoldMetrics>::new(),
            chair_metrics: ChairMetrics {
                approve_candidate_count,
                reduce_size_count: 0,
                no_trade_count: no_trade,
                require_confirm_count: 0,
                groupthink_risk_avg: 0.0,
                disagreement_score_avg: 0.0,
                cluster_penalty_avg: 0.0,
            },
            reason_codes: vec![],
        }],
        aggregate_metrics: WalkForwardAggregateMetrics {
            trade_metrics: TradeMetrics {
                total_trades: 3,
                wins: 2,
                losses: 1,
                neutrals: 0,
                win_rate: 0.66,
                avg_win_pct: 0.02,
                avg_loss_pct: -0.01,
                gross_return_pct: 0.03,
                net_return_pct: 0.02,
                profit_factor: Some(1.5),
                max_drawdown_pct,
                avg_bars_held: 2.0,
                reason_codes: vec![],
            },
            decision_metrics,
            no_trade_metrics: NoTradeMetrics {
                no_trade_count: no_trade,
                avoided_loss_count: 2,
                missed_gain_count: 1,
                avg_avoided_loss_score: 0.2,
                avg_missed_gain_penalty: -0.1,
                net_silence_value: 0.1,
            },
            risk_metrics,
            calibration_metrics: CalibrationMetrics {
                brier_score: 0.1,
                calibration_bins: vec![],
                expected_calibration_error: Some(0.03),
            },
            regime_metrics: Vec::<RegimeMetrics>::new(),
            persona_metrics: Vec::<PersonaFoldMetrics>::new(),
            chair_metrics: ChairMetrics {
                approve_candidate_count,
                reduce_size_count: 0,
                no_trade_count: no_trade,
                require_confirm_count: 0,
                groupthink_risk_avg: 0.0,
                disagreement_score_avg: 0.0,
                cluster_penalty_avg: 0.0,
            },
        },
        feature_schema: FeatureSchema::from_feature_names(&[]),
        reason_codes: vec![],
    }
}

#[test]
fn denials_are_counted_correctly() {
    let report = RiskAiInteractionReport::from_walk_forward_report(
        "model",
        &walk_forward_report(4, 2, 3, 0.5, 0.05),
    );
    assert_eq!(report.denied_by_risk, 4);
    assert_eq!(report.approved_candidates, 3);
}

#[test]
fn high_denial_with_defensive_value_is_not_automatically_warned() {
    let report = RiskAiInteractionReport::from_walk_forward_report(
        "model",
        &walk_forward_report(10, 0, 0, 1.2, 0.05),
    );
    assert!(report.warnings.is_empty());
}

#[test]
fn low_denial_with_high_drawdown_warns() {
    let report = RiskAiInteractionReport::from_walk_forward_report(
        "model",
        &walk_forward_report(0, 0, 9, 0.0, 0.30),
    );
    assert!(!report.warnings.is_empty());
}
