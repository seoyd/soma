use soma_zero::{
    AttributionRecord, CounterfactualRole, DecisionRecord, Horizon, OutcomeRecord, PersonaTier,
    ReasonCode, Regime, RiskDecision, RiskDecisionKind, ShadowOutcomeRecord, Side, SignalOutput,
    Stance, TradeProposal, TripleBarrierOutcome, TripleBarrierResult, active_persona_cards,
    build_persona_evaluation_inputs,
};
use soma_zero::{ChairDecisionKind, ChairOutput, InvestorVote, SixPrinciples};

fn signal(confidence: f64) -> SignalOutput {
    SignalOutput {
        symbol: "TEST".to_string(),
        horizon_bars: 8,
        p_win: 0.60,
        p_stop: 0.30,
        expected_return: 0.02,
        expected_drawdown: 0.01,
        confidence,
        no_trade_probability: 0.20,
        source: "test".to_string(),
    }
}

fn principles() -> SixPrinciples {
    SixPrinciples {
        signal_edge: 0.7,
        regime_fit: 0.8,
        liquidity_fit: 0.8,
        loss_protection: 0.7,
        event_risk: 0.6,
        execution_quality: 0.8,
    }
}

fn vote(persona_id: &str, stance: Stance) -> InvestorVote {
    InvestorVote {
        persona_id: persona_id.to_string(),
        cluster_id: "test".to_string(),
        stance,
        conviction: 0.8,
        voice_power: 0.7,
        veto: false,
        six_principles: principles(),
        expected_return_adjustment: 0.0,
        risk_penalty: 0.1,
        reason_codes: vec![],
    }
}

fn chair_output() -> ChairOutput {
    ChairOutput {
        selected_speakers: vec!["momentum_trend_fast".to_string()],
        lead_speaker: "momentum_trend_fast".to_string(),
        forced_contrarian: false,
        council_score: 0.4,
        disagreement_score: 0.1,
        groupthink_risk: 0.2,
        size_multiplier: 0.5,
        decision: ChairDecisionKind::ApproveCandidate,
        reason_codes: vec![ReasonCode::CandidateApproved],
    }
}

fn risk_decision(kind: RiskDecisionKind) -> RiskDecision {
    RiskDecision {
        kind,
        approved_order_plan: None,
        reason_codes: match kind {
            RiskDecisionKind::Deny => vec![ReasonCode::ExpectedEdgeNonPositive],
            RiskDecisionKind::ApprovePaper => vec![ReasonCode::ApprovePaperOnly],
            RiskDecisionKind::Cooldown => vec![ReasonCode::ConsecutiveLossGateBreached],
            RiskDecisionKind::EmergencyStop => vec![ReasonCode::DailyLossGateBreached],
        },
        audit_id: "audit-1".to_string(),
    }
}

fn proposal() -> TradeProposal {
    TradeProposal {
        symbol: "TEST".to_string(),
        side: Side::Long,
        quantity_hint: 0.5,
        entry_price_hint: 100.0,
        stop_loss: Some(99.0),
        take_profit: Some(102.0),
        max_slippage_bps: 2.0,
        expected_edge_after_cost: 0.01,
        confidence: 0.8,
        source_chair_output: chair_output(),
    }
}

fn barrier_result(outcome: TripleBarrierOutcome, net_return_pct: f64) -> TripleBarrierResult {
    TripleBarrierResult {
        outcome,
        first_hit: match outcome {
            TripleBarrierOutcome::Win => soma_zero::BarrierHit::TakeProfit,
            TripleBarrierOutcome::Loss => soma_zero::BarrierHit::StopLoss,
            TripleBarrierOutcome::Neutral => soma_zero::BarrierHit::TimeExpired,
            TripleBarrierOutcome::NoData => soma_zero::BarrierHit::NoData,
        },
        entry_index: 0,
        exit_index: 1,
        entry_price: 100.0,
        exit_price: 99.0,
        gross_return_pct: net_return_pct,
        net_return_pct,
        max_favorable_excursion_pct: 0.02,
        max_adverse_excursion_pct: 0.01,
        bars_held: 1,
        reason_codes: vec![],
    }
}

fn decision_record(id: &str) -> DecisionRecord {
    DecisionRecord {
        id: id.to_string(),
        timestamp_ms: 1,
        symbol: "TEST".to_string(),
        signal_output: signal(0.9),
        investor_votes: vec![
            vote("momentum_trend_fast", Stance::Buy),
            vote("value_quality_filter", Stance::NoTrade),
        ],
        chair_output: chair_output(),
        risk_decision: risk_decision(RiskDecisionKind::ApprovePaper),
        trade_proposal: Some(proposal()),
        selected_for_execution: true,
        paper_order_id: Some("paper-000001".to_string()),
        reason_codes: vec![ReasonCode::CandidateApproved],
        audit_event_id: "audit-1".to_string(),
    }
}

fn attribution(persona_id: &str, role: CounterfactualRole) -> AttributionRecord {
    AttributionRecord {
        persona_id: persona_id.to_string(),
        selected_for_decision: true,
        stance: Stance::Buy,
        conviction: 0.8,
        voice_power: 0.7,
        contribution_score: 0.5,
        counterfactual_role: role,
        reason_codes: vec![],
    }
}

#[test]
fn decision_record_links_to_outcome_record() {
    let decision = decision_record("decision-1");
    let outcome = OutcomeRecord {
        decision_id: decision.id.clone(),
        symbol: "TEST".to_string(),
        timestamp_ms: 1,
        regime: Regime::TrendUp,
        horizon: Horizon::Intraday,
        signal_confidence: 0.9,
        executed: true,
        denied_by_risk: false,
        no_trade: false,
        triple_barrier_result: Some(barrier_result(TripleBarrierOutcome::Win, 0.02)),
        hypothetical_result: None,
        realized_net_return_pct: 0.02,
        avoided_loss_score: 0.0,
        missed_gain_penalty: 0.0,
        attribution_records: vec![attribution(
            "momentum_trend_fast",
            CounterfactualRole::SupportedFinalDecision,
        )],
        shadow_outcomes: vec![],
        reason_codes: vec![],
    };
    assert_eq!(decision.id, outcome.decision_id);
}

#[test]
fn executed_trade_record_keeps_triple_barrier_result() {
    let outcome = OutcomeRecord {
        decision_id: "decision-2".to_string(),
        symbol: "TEST".to_string(),
        timestamp_ms: 1,
        regime: Regime::TrendUp,
        horizon: Horizon::Intraday,
        signal_confidence: 0.8,
        executed: true,
        denied_by_risk: false,
        no_trade: false,
        triple_barrier_result: Some(barrier_result(TripleBarrierOutcome::Win, 0.02)),
        hypothetical_result: None,
        realized_net_return_pct: 0.02,
        avoided_loss_score: 0.0,
        missed_gain_penalty: 0.0,
        attribution_records: vec![],
        shadow_outcomes: vec![],
        reason_codes: vec![],
    };
    assert_eq!(
        outcome.triple_barrier_result.unwrap().outcome,
        TripleBarrierOutcome::Win
    );
}

#[test]
fn no_trade_record_keeps_hypothetical_result() {
    let outcome = OutcomeRecord {
        decision_id: "decision-3".to_string(),
        symbol: "TEST".to_string(),
        timestamp_ms: 1,
        regime: Regime::Range,
        horizon: Horizon::Intraday,
        signal_confidence: 0.6,
        executed: false,
        denied_by_risk: false,
        no_trade: true,
        triple_barrier_result: None,
        hypothetical_result: Some(barrier_result(TripleBarrierOutcome::Loss, -0.02)),
        realized_net_return_pct: 0.0,
        avoided_loss_score: 0.014,
        missed_gain_penalty: 0.0,
        attribution_records: vec![],
        shadow_outcomes: vec![],
        reason_codes: vec![ReasonCode::NoTradeCounterfactual],
    };
    assert!(outcome.no_trade);
    assert!(outcome.hypothetical_result.is_some());
}

#[test]
fn risk_denied_record_keeps_hypothetical_result_without_execution() {
    let outcome = OutcomeRecord {
        decision_id: "decision-4".to_string(),
        symbol: "TEST".to_string(),
        timestamp_ms: 1,
        regime: Regime::TrendUp,
        horizon: Horizon::Intraday,
        signal_confidence: 0.7,
        executed: false,
        denied_by_risk: true,
        no_trade: false,
        triple_barrier_result: None,
        hypothetical_result: Some(barrier_result(TripleBarrierOutcome::Loss, -0.03)),
        realized_net_return_pct: 0.0,
        avoided_loss_score: 0.021,
        missed_gain_penalty: 0.0,
        attribution_records: vec![],
        shadow_outcomes: vec![],
        reason_codes: vec![ReasonCode::RiskDeniedCounterfactual],
    };
    assert!(outcome.denied_by_risk);
    assert!(!outcome.executed);
    assert!(outcome.hypothetical_result.is_some());
}

#[test]
fn attribution_record_keeps_persona_and_role() {
    let record = attribution("cycle_risk_skeptic", CounterfactualRole::RiskVetoAligned);
    assert_eq!(record.persona_id, "cycle_risk_skeptic");
    assert_eq!(
        record.counterfactual_role,
        CounterfactualRole::RiskVetoAligned
    );
}

#[test]
fn persona_evaluation_inputs_aggregate_records_by_persona() {
    let profiles = active_persona_cards();
    let outcomes = vec![
        OutcomeRecord {
            decision_id: "decision-a".to_string(),
            symbol: "TEST".to_string(),
            timestamp_ms: 1,
            regime: Regime::TrendUp,
            horizon: Horizon::Intraday,
            signal_confidence: 0.92,
            executed: true,
            denied_by_risk: false,
            no_trade: false,
            triple_barrier_result: Some(barrier_result(TripleBarrierOutcome::Loss, -0.02)),
            hypothetical_result: None,
            realized_net_return_pct: -0.02,
            avoided_loss_score: 0.0,
            missed_gain_penalty: 0.0,
            attribution_records: vec![attribution(
                "momentum_trend_fast",
                CounterfactualRole::SupportedFinalDecision,
            )],
            shadow_outcomes: vec![ShadowOutcomeRecord {
                persona_id: "value_quality_filter".to_string(),
                hypothetical_stance: Stance::NoTrade,
                hypothetical_result: None,
                would_have_supported_trade: false,
                would_have_blocked_trade: true,
                evaluation_pending: false,
            }],
            reason_codes: vec![],
        },
        OutcomeRecord {
            decision_id: "decision-b".to_string(),
            symbol: "TEST".to_string(),
            timestamp_ms: 2,
            regime: Regime::Range,
            horizon: Horizon::Intraday,
            signal_confidence: 0.60,
            executed: false,
            denied_by_risk: false,
            no_trade: true,
            triple_barrier_result: None,
            hypothetical_result: Some(barrier_result(TripleBarrierOutcome::Loss, -0.03)),
            realized_net_return_pct: 0.0,
            avoided_loss_score: 0.021,
            missed_gain_penalty: 0.0,
            attribution_records: vec![attribution(
                "momentum_trend_fast",
                CounterfactualRole::OpposedFinalDecision,
            )],
            shadow_outcomes: vec![],
            reason_codes: vec![ReasonCode::NoTradeCounterfactual],
        },
    ];

    let inputs = build_persona_evaluation_inputs(&outcomes, &profiles);
    let momentum = inputs
        .iter()
        .find(|input| input.persona_id == "momentum_trend_fast")
        .expect("momentum profile");
    assert_eq!(momentum.sample_count, 2);
    assert_eq!(momentum.high_confidence_miss_count, 1);
    assert!(momentum.survival_score_components.silence_value > 0.5);
}

#[test]
fn persona_evaluation_inputs_keep_profile_metadata() {
    let profiles = active_persona_cards();
    let inputs = build_persona_evaluation_inputs(&[], &profiles);
    let cycle = inputs
        .iter()
        .find(|input| input.persona_id == "cycle_risk_skeptic")
        .expect("cycle profile");
    assert_eq!(cycle.tier, PersonaTier::A);
    assert_eq!(cycle.horizon, Horizon::Swing);
}
