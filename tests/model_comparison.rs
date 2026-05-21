use soma_zero::{
    CalibrationReport, FoldReport, ReasonCode, TradeMetrics, WalkForwardAggregateMetrics,
    WalkForwardConfig, WalkForwardReport, compare_walk_forward_reports,
};

fn trade_metrics(
    net: f64,
    drawdown: f64,
    trades: usize,
    profit_factor: Option<f64>,
) -> TradeMetrics {
    TradeMetrics {
        total_trades: trades,
        wins: trades / 2,
        losses: trades / 2,
        neutrals: 0,
        win_rate: 0.5,
        avg_win_pct: 0.02,
        avg_loss_pct: -0.01,
        gross_return_pct: net,
        net_return_pct: net,
        profit_factor,
        max_drawdown_pct: drawdown,
        avg_bars_held: 3.0,
        reason_codes: vec![],
    }
}

fn report(net: f64, drawdown: f64, profit_factor: Option<f64>) -> WalkForwardReport {
    WalkForwardReport {
        symbol: "CMP".to_string(),
        timeframe: soma_zero::Timeframe::FiveMinute,
        config: WalkForwardConfig::default(),
        folds: Vec::<FoldReport>::new(),
        aggregate_metrics: WalkForwardAggregateMetrics {
            trade_metrics: trade_metrics(net, drawdown, 4, profit_factor),
            decision_metrics: soma_zero::DecisionMetrics {
                total_decisions: 4,
                executed: 4,
                denied_by_risk: 0,
                no_trade: 0,
                require_confirm_count: 0,
                approve_candidate_count: 4,
                reason_code_counts: Default::default(),
            },
            no_trade_metrics: soma_zero::NoTradeMetrics {
                no_trade_count: 0,
                avoided_loss_count: 0,
                missed_gain_count: 0,
                avg_avoided_loss_score: 0.0,
                avg_missed_gain_penalty: 0.0,
                net_silence_value: 0.0,
            },
            risk_metrics: soma_zero::RiskGovernorMetrics {
                denied_count: 0,
                emergency_stop_count: 0,
                cooldown_count: 0,
                avoided_loss_count: 0,
                missed_gain_count: 0,
                defensive_value: 0.0,
                opportunity_cost: 0.0,
            },
            calibration_metrics: soma_zero::CalibrationMetrics {
                brier_score: 0.1,
                calibration_bins: vec![],
                expected_calibration_error: Some(0.0),
            },
            regime_metrics: vec![],
            persona_metrics: vec![],
            chair_metrics: soma_zero::ChairMetrics {
                approve_candidate_count: 4,
                reduce_size_count: 0,
                no_trade_count: 0,
                require_confirm_count: 0,
                groupthink_risk_avg: 0.0,
                disagreement_score_avg: 0.0,
                cluster_penalty_avg: 0.0,
            },
        },
        feature_schema: soma_zero::FeatureSchema {
            schema_version: 1,
            feature_names: vec![],
            feature_count: 0,
            checksum: 1,
            created_by: "test".to_string(),
        },
        reason_codes: vec![],
    }
}

#[test]
fn external_better_requires_more_than_net_return() {
    let baseline = report(0.05, 0.04, Some(1.5));
    let external = report(0.06, 0.12, Some(1.6));
    let comparison = compare_walk_forward_reports(
        "baseline", "external", &baseline, &external, None, None, None,
    );

    assert!(!comparison.external_better);
    assert!(
        comparison
            .reason_codes
            .contains(&ReasonCode::ComparisonNotConclusive)
    );
}

#[test]
fn model_comparison_is_deterministic() {
    let baseline = report(0.05, 0.04, Some(1.5));
    let external = report(0.08, 0.03, Some(1.8));
    let calibration = CalibrationReport {
        model_id: "external".to_string(),
        fold_id: None,
        total_count: 4,
        brier_score: 0.08,
        expected_calibration_error: 0.01,
        bins: vec![],
        reason_codes: vec![],
    };
    let left = compare_walk_forward_reports(
        "baseline",
        "external",
        &baseline,
        &external,
        None,
        Some(&calibration),
        None,
    );
    let right = compare_walk_forward_reports(
        "baseline",
        "external",
        &baseline,
        &external,
        None,
        Some(&calibration),
        None,
    );

    assert_eq!(left, right);
}
