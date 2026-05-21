use soma_zero::backtest::simulate_paper_cycle;
use soma_zero::chair::{ChairConfig, ChairEngine};
use soma_zero::core::{
    ChairDecisionKind, ChairInput, MarketSnapshot, ReasonCode, Regime, RiskDecisionKind,
    RiskSnapshot, SignalOutput, TradeProposal,
};
use soma_zero::league::{
    CycleRiskSkeptic, HypotheticalTradeOutcome, MomentumTrendFast, Persona, SurvivalComponents,
    ValueQualityFilter, default_league_votes, silence_value, update_voice_power, violation_outcome,
};
use soma_zero::paper::{Broker, PaperBroker};
use soma_zero::risk::RiskGovernor;
use soma_zero::signal::MockSignalEngine;

fn market() -> MarketSnapshot {
    MarketSnapshot {
        symbol: "ETHUSD".to_string(),
        timestamp_ms: 1_715_000_000_000,
        price: 100.0,
        bid: 99.99,
        ask: 100.01,
        spread_bps: 2.0,
        volume: 15_000.0,
        trade_value: 1_500_000.0,
        volatility: 0.012,
        regime: Regime::TrendUp,
        data_quality_score: 0.98,
    }
}

fn risk_snapshot() -> RiskSnapshot {
    RiskSnapshot {
        daily_pnl_pct: 0.0,
        consecutive_losses: 0,
        current_positions_count: 0,
        total_exposure_pct: 0.0,
        symbol_exposure_pct: 0.0,
        api_health_score: 1.0,
        data_quality_score: 0.98,
    }
}

fn signal() -> SignalOutput {
    MockSignalEngine::default().evaluate(&market())
}

fn proposal() -> TradeProposal {
    let chair = ChairEngine::default();
    let signal = signal();
    let votes = default_league_votes(&market(), &signal);
    let input = ChairInput {
        market: market(),
        signal: signal.clone(),
        votes,
        full_auto: false,
    };
    let chair_output = chair.evaluate(&input);
    chair
        .build_trade_proposal(&market(), &signal, &chair_output)
        .expect("proposal")
}

#[test]
fn risk_governor_denies_by_default() {
    let governor = RiskGovernor::default();
    let decision = governor.evaluate(&market(), &risk_snapshot(), None, market().timestamp_ms);
    assert_eq!(decision.kind, RiskDecisionKind::Deny);
    assert!(decision.reason_codes.contains(&ReasonCode::DeniedByDefault));
}

#[test]
fn risk_governor_denies_if_expected_edge_non_positive() {
    let governor = RiskGovernor::default();
    let mut proposal = proposal();
    proposal.expected_edge_after_cost = 0.0;
    let decision = governor.evaluate(
        &market(),
        &risk_snapshot(),
        Some(&proposal),
        market().timestamp_ms,
    );
    assert_eq!(decision.kind, RiskDecisionKind::Deny);
    assert!(
        decision
            .reason_codes
            .contains(&ReasonCode::ExpectedEdgeNonPositive)
    );
}

#[test]
fn risk_governor_denies_if_stop_loss_missing() {
    let governor = RiskGovernor::default();
    let mut proposal = proposal();
    proposal.stop_loss = None;
    let decision = governor.evaluate(
        &market(),
        &risk_snapshot(),
        Some(&proposal),
        market().timestamp_ms,
    );
    assert_eq!(decision.kind, RiskDecisionKind::Deny);
    assert!(decision.reason_codes.contains(&ReasonCode::MissingStopLoss));
}

#[test]
fn risk_governor_emergency_stops_on_daily_loss_breach() {
    let governor = RiskGovernor::default();
    let mut snapshot = risk_snapshot();
    snapshot.daily_pnl_pct = -0.05;
    let decision = governor.evaluate(
        &market(),
        &snapshot,
        Some(&proposal()),
        market().timestamp_ms,
    );
    assert_eq!(decision.kind, RiskDecisionKind::EmergencyStop);
    assert!(
        decision
            .reason_codes
            .contains(&ReasonCode::DailyLossGateBreached)
    );
}

#[test]
fn consecutive_losses_trigger_cooldown() {
    let governor = RiskGovernor::default();
    let mut snapshot = risk_snapshot();
    snapshot.consecutive_losses = 3;
    let decision = governor.evaluate(
        &market(),
        &snapshot,
        Some(&proposal()),
        market().timestamp_ms,
    );
    assert_eq!(decision.kind, RiskDecisionKind::Cooldown);
    assert!(
        decision
            .reason_codes
            .contains(&ReasonCode::ConsecutiveLossGateBreached)
    );
}

#[test]
fn unknown_regime_denies_or_no_trades() {
    let governor = RiskGovernor::default();
    let mut uncertain_market = market();
    uncertain_market.regime = Regime::Unknown;
    let decision = governor.evaluate(
        &uncertain_market,
        &risk_snapshot(),
        Some(&proposal()),
        uncertain_market.timestamp_ms,
    );
    assert_eq!(decision.kind, RiskDecisionKind::Deny);
    assert!(
        decision
            .reason_codes
            .contains(&ReasonCode::UnknownRegimeGateBreached)
    );
}

#[test]
fn low_data_quality_denies() {
    let governor = RiskGovernor::default();
    let mut low_quality_market = market();
    low_quality_market.data_quality_score = 0.40;
    let decision = governor.evaluate(
        &low_quality_market,
        &risk_snapshot(),
        Some(&proposal()),
        low_quality_market.timestamp_ms,
    );
    assert_eq!(decision.kind, RiskDecisionKind::Deny);
    assert!(
        decision
            .reason_codes
            .contains(&ReasonCode::DataQualityGateBreached)
    );
}

#[test]
fn chair_degrades_require_confirm_to_no_trade_in_full_auto_mode() {
    let chair = ChairEngine {
        config: ChairConfig {
            strong_threshold: 0.80,
            weak_threshold: 0.18,
            ..ChairConfig::default()
        },
    };
    let signal = signal();
    let votes = vec![
        MomentumTrendFast::default().vote(&market(), &signal),
        ValueQualityFilter::default().vote(&market(), &signal),
        CycleRiskSkeptic { base_voice: 0.10 }.vote(&market(), &signal),
    ];
    let output = chair.evaluate(&ChairInput {
        market: market(),
        signal,
        votes,
        full_auto: true,
    });
    assert_eq!(output.decision, ChairDecisionKind::NoTrade);
    assert!(
        output
            .reason_codes
            .contains(&ReasonCode::RequireConfirmBlockedInAuto)
    );
}

#[test]
fn chair_applies_contrarian_inclusion_when_votes_are_one_sided() {
    let chair = ChairEngine::default();
    let signal_output = signal();
    let mut lead = MomentumTrendFast::default().vote(&market(), &signal_output);
    lead.voice_power = 0.72;
    lead.conviction = 0.88;
    let mut wingman = lead.clone();
    wingman.persona_id = "momentum_shadow".to_string();
    wingman.voice_power = 0.58;
    wingman.conviction = 0.76;
    let mut contrarian = CycleRiskSkeptic::default().vote(&market(), &signal_output);
    contrarian.persona_id = "cycle_shadow".to_string();
    contrarian.voice_power = 0.26;
    contrarian.conviction = 0.42;
    let output = chair.evaluate(&ChairInput {
        market: market(),
        signal: signal_output,
        votes: vec![lead, wingman, contrarian],
        full_auto: false,
    });
    assert!(output.forced_contrarian);
    assert!(
        output
            .reason_codes
            .contains(&ReasonCode::ContrarianIncluded)
    );
}

#[test]
fn chair_applies_cluster_penalty() {
    let chair = ChairEngine::default();
    let signal_output = signal();
    let mut duplicate = MomentumTrendFast::default().vote(&market(), &signal_output);
    duplicate.persona_id = "momentum_clone".to_string();
    duplicate.voice_power = 0.7;
    duplicate.conviction = 0.8;
    let output = chair.evaluate(&ChairInput {
        market: market(),
        signal: signal_output.clone(),
        votes: vec![
            MomentumTrendFast::default().vote(&market(), &signal_output),
            duplicate,
            CycleRiskSkeptic::default().vote(&market(), &signal_output),
        ],
        full_auto: false,
    });
    assert!(
        output
            .reason_codes
            .contains(&ReasonCode::ClusterPenaltyApplied)
    );
}

#[test]
fn no_trade_receives_positive_silence_value_when_stop_would_have_hit() {
    let value = silence_value(HypotheticalTradeOutcome::StopFirst(0.10));
    assert!((value - 0.07).abs() < 1e-12);
}

#[test]
fn doctrine_violation_forces_quarantine() {
    let tier = violation_outcome(soma_zero::PersonaTier::A, true, false);
    assert_eq!(tier, soma_zero::PersonaTier::XQuarantined);
}

#[test]
fn voice_update_is_stable_and_bounded() {
    let mut voice = 0.5;
    for _ in 0..100 {
        voice = update_voice_power(voice, 1.0);
    }
    assert!((0.0..=1.0).contains(&voice));
    assert!(voice <= 1.0);
}

#[test]
fn paper_broker_never_calls_real_broker() {
    let signal_engine = MockSignalEngine::default();
    let chair = ChairEngine::default();
    let governor = RiskGovernor::default();
    let mut broker = PaperBroker::default();
    let result = simulate_paper_cycle(
        &market(),
        &risk_snapshot(),
        &signal_engine,
        &chair,
        &governor,
        &mut broker,
        false,
    );
    assert!(!broker.supports_live_execution());
    assert_eq!(broker.live_call_count(), 0);
    if let Some(order) = result.paper_order {
        assert!(order.paper_only);
    }
}

#[test]
fn all_decisions_produce_reason_codes() {
    let signal_engine = MockSignalEngine::default();
    let chair = ChairEngine::default();
    let governor = RiskGovernor::default();
    let mut broker = PaperBroker::default();
    let result = simulate_paper_cycle(
        &market(),
        &risk_snapshot(),
        &signal_engine,
        &chair,
        &governor,
        &mut broker,
        false,
    );
    assert!(!result.chair_output.reason_codes.is_empty());
    assert!(!result.risk_decision.reason_codes.is_empty());
    assert!(
        result
            .audit_events
            .iter()
            .all(|event| !event.reason_codes.is_empty())
    );
}

#[test]
fn same_input_produces_same_output() {
    let signal_engine = MockSignalEngine::default();
    let chair = ChairEngine::default();
    let governor = RiskGovernor::default();
    let mut broker_a = PaperBroker::default();
    let mut broker_b = PaperBroker::default();
    let result_a = simulate_paper_cycle(
        &market(),
        &risk_snapshot(),
        &signal_engine,
        &chair,
        &governor,
        &mut broker_a,
        false,
    );
    let result_b = simulate_paper_cycle(
        &market(),
        &risk_snapshot(),
        &signal_engine,
        &chair,
        &governor,
        &mut broker_b,
        false,
    );
    assert_eq!(result_a, result_b);
}

#[test]
fn survival_score_uses_required_formula_shape() {
    let score = soma_zero::survival_score(SurvivalComponents {
        drawdown_control: 0.8,
        risk_efficiency: 0.7,
        net_expectancy_after_cost: 0.6,
        calibration: 0.5,
        regime_fit: 0.7,
        silence_value: 0.4,
        doctrine_consistency: 0.9,
        overconfidence_penalty: 0.1,
        overtrade_penalty: 0.1,
        correlation_penalty: 0.05,
        doctrine_violation_penalty: 0.0,
    });
    assert!(score.is_finite());
}
