use soma_zero::{
    BarrierHit, ChairDecisionKind, ChairOutput, DecisionRecord, Horizon, InvestorVote,
    OutcomeRecord, Regime, RiskDecision, RiskDecisionKind, SignalOutput, TripleBarrierOutcome,
    TripleBarrierResult, compute_calibration_metrics, compute_decision_metrics,
    compute_no_trade_metrics, compute_regime_metrics, compute_risk_metrics, compute_trade_metrics,
};

fn decision(
    id: &str,
    p_win: f64,
    chair_decision: ChairDecisionKind,
    risk_kind: RiskDecisionKind,
) -> DecisionRecord {
    DecisionRecord {
        id: id.to_string(),
        timestamp_ms: 0,
        symbol: "MET".to_string(),
        signal_output: SignalOutput {
            symbol: "MET".to_string(),
            horizon_bars: 8,
            p_win,
            p_stop: 0.3,
            expected_return: 0.02,
            expected_drawdown: 0.01,
            confidence: 0.7,
            no_trade_probability: 0.2,
            source: "baseline_rule_v0".to_string(),
        },
        investor_votes: Vec::<InvestorVote>::new(),
        chair_output: ChairOutput {
            selected_speakers: vec![],
            lead_speaker: "chair".to_string(),
            forced_contrarian: false,
            council_score: 0.6,
            disagreement_score: 0.2,
            groupthink_risk: 0.1,
            size_multiplier: 1.0,
            decision: chair_decision,
            reason_codes: vec![],
        },
        risk_decision: RiskDecision {
            kind: risk_kind,
            approved_order_plan: None,
            reason_codes: vec![],
            audit_id: "audit".to_string(),
        },
        trade_proposal: None,
        selected_for_execution: risk_kind == RiskDecisionKind::ApprovePaper,
        paper_order_id: None,
        reason_codes: vec![],
        audit_event_id: "audit".to_string(),
    }
}

fn barrier(
    outcome: TripleBarrierOutcome,
    net: f64,
    gross: f64,
    bars: usize,
) -> TripleBarrierResult {
    TripleBarrierResult {
        outcome,
        first_hit: match outcome {
            TripleBarrierOutcome::Win => BarrierHit::TakeProfit,
            TripleBarrierOutcome::Loss => BarrierHit::StopLoss,
            TripleBarrierOutcome::Neutral => BarrierHit::TimeExpired,
            TripleBarrierOutcome::NoData => BarrierHit::NoData,
        },
        entry_index: 0,
        exit_index: bars,
        entry_price: 100.0,
        exit_price: 101.0,
        gross_return_pct: gross,
        net_return_pct: net,
        max_favorable_excursion_pct: 0.0,
        max_adverse_excursion_pct: 0.0,
        bars_held: bars,
        reason_codes: vec![],
    }
}

fn outcome(
    id: &str,
    regime: Regime,
    executed: bool,
    denied: bool,
    no_trade: bool,
    result: Option<TripleBarrierResult>,
    avoided: f64,
    penalty: f64,
) -> OutcomeRecord {
    OutcomeRecord {
        decision_id: id.to_string(),
        symbol: "MET".to_string(),
        timestamp_ms: 0,
        regime,
        horizon: Horizon::Intraday,
        signal_confidence: 0.8,
        executed,
        denied_by_risk: denied,
        no_trade,
        realized_net_return_pct: result.as_ref().map(|row| row.net_return_pct).unwrap_or(0.0),
        triple_barrier_result: result,
        hypothetical_result: None,
        avoided_loss_score: avoided,
        missed_gain_penalty: penalty,
        attribution_records: vec![],
        shadow_outcomes: vec![],
        reason_codes: vec![],
    }
}

#[test]
fn trade_metrics_and_profit_factor_are_correct() {
    let outcomes = vec![
        outcome(
            "d1",
            Regime::TrendUp,
            true,
            false,
            false,
            Some(barrier(TripleBarrierOutcome::Win, 0.03, 0.035, 3)),
            0.0,
            0.0,
        ),
        outcome(
            "d2",
            Regime::TrendUp,
            true,
            false,
            false,
            Some(barrier(TripleBarrierOutcome::Loss, -0.01, -0.006, 2)),
            0.0,
            0.0,
        ),
        outcome(
            "d3",
            Regime::Range,
            true,
            false,
            false,
            Some(barrier(TripleBarrierOutcome::Neutral, 0.0, 0.001, 4)),
            0.0,
            0.0,
        ),
    ];
    let metrics = compute_trade_metrics(&outcomes);

    assert_eq!(metrics.total_trades, 3);
    assert_eq!(metrics.wins, 1);
    assert_eq!(metrics.losses, 1);
    assert!((metrics.win_rate - (1.0 / 3.0)).abs() < 1e-9);
    assert_eq!(metrics.profit_factor, Some(3.0));
    assert!(metrics.max_drawdown_pct >= 0.0);
}

#[test]
fn no_trade_and_risk_metrics_credit_defensive_behavior() {
    let outcomes = vec![
        outcome("d1", Regime::RiskOff, false, false, true, None, 0.04, 0.0),
        outcome(
            "d2",
            Regime::RiskOff,
            false,
            true,
            false,
            None,
            0.03,
            -0.005,
        ),
    ];
    let decisions = vec![
        decision(
            "d1",
            0.55,
            ChairDecisionKind::NoTrade,
            RiskDecisionKind::Deny,
        ),
        decision(
            "d2",
            0.60,
            ChairDecisionKind::ApproveCandidate,
            RiskDecisionKind::Deny,
        ),
    ];

    let no_trade = compute_no_trade_metrics(&outcomes);
    let risk = compute_risk_metrics(&decisions, &outcomes);

    assert_eq!(no_trade.avoided_loss_count, 1);
    assert_eq!(no_trade.missed_gain_count, 0);
    assert!(no_trade.net_silence_value > 0.0);
    assert_eq!(risk.denied_count, 1);
    assert_eq!(risk.avoided_loss_count, 1);
    assert!(risk.defensive_value > 0.0);
    assert!(risk.opportunity_cost > 0.0);
}

#[test]
fn calibration_and_regime_metrics_are_grouped_correctly() {
    let decisions = vec![
        decision(
            "d1",
            0.8,
            ChairDecisionKind::ApproveCandidate,
            RiskDecisionKind::ApprovePaper,
        ),
        decision(
            "d2",
            0.2,
            ChairDecisionKind::ApproveCandidate,
            RiskDecisionKind::ApprovePaper,
        ),
    ];
    let outcomes = vec![
        outcome(
            "d1",
            Regime::TrendUp,
            true,
            false,
            false,
            Some(barrier(TripleBarrierOutcome::Win, 0.02, 0.025, 2)),
            0.0,
            0.0,
        ),
        outcome(
            "d2",
            Regime::Panic,
            true,
            false,
            false,
            Some(barrier(TripleBarrierOutcome::Loss, -0.01, -0.008, 2)),
            0.0,
            0.0,
        ),
    ];

    let calibration = compute_calibration_metrics(&decisions, &outcomes);
    let regimes = compute_regime_metrics(&decisions, &outcomes);
    let decision_metrics = compute_decision_metrics(&decisions, &outcomes);

    assert!((calibration.brier_score - 0.04).abs() < 1e-9);
    assert_eq!(regimes.len(), 2);
    assert_eq!(decision_metrics.total_decisions, 2);
}
