use soma_zero::{
    ChairCommitteeConfig, ChairV0, CommitteeDecisionRecord, CommitteeFinalAction, CommitteeInput,
    CommitteeRiskBridge, EvidenceSourceKind, MarketSnapshot, PersonaHorizon, PersonaScoringInput,
    PersonaStance, PersonaVote, ProviderMarket, ReasonCode, Regime, RiskDecisionKind, RiskSnapshot,
    SignalOutput,
};

fn market_snapshot(data_quality_score: f64) -> MarketSnapshot {
    MarketSnapshot {
        symbol: "BTC-KRW".to_string(),
        timestamp_ms: 1_700_000_000_000,
        price: 100_000_000.0,
        bid: 99_950_000.0,
        ask: 100_050_000.0,
        spread_bps: 8.0,
        volume: 10_000.0,
        trade_value: 1_000_000.0,
        volatility: 0.02,
        regime: Regime::TrendUp,
        data_quality_score,
    }
}

fn risk_snapshot(data_quality_score: f64) -> RiskSnapshot {
    RiskSnapshot {
        daily_pnl_pct: 0.0,
        consecutive_losses: 0,
        current_positions_count: 0,
        total_exposure_pct: 0.0,
        symbol_exposure_pct: 0.0,
        api_health_score: 1.0,
        data_quality_score,
    }
}

fn scoring_input(expected_edge_after_cost: f64, expected_drawdown: f64) -> PersonaScoringInput {
    PersonaScoringInput {
        symbol: "BTC-KRW".to_string(),
        timestamp_ms: 1_700_000_000_000,
        source_kind: EvidenceSourceKind::OfficialApiCollected,
        market: ProviderMarket::Crypto,
        target_horizon: PersonaHorizon::Swing,
        feature_vector: None,
        regime: Regime::TrendUp,
        signal_output: SignalOutput {
            symbol: "BTC-KRW".to_string(),
            horizon_bars: 12,
            p_win: 0.62,
            p_stop: 0.28,
            expected_return: expected_edge_after_cost,
            expected_drawdown,
            confidence: 0.82,
            no_trade_probability: 0.18,
            source: "test".to_string(),
        },
        data_quality_score: 0.95,
        spread_bps: Some(8.0),
        expected_edge_after_cost,
        expected_drawdown,
        risk_snapshot: Some(risk_snapshot(0.95)),
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn approved_record() -> CommitteeDecisionRecord {
    let input = CommitteeInput {
        scoring_input: scoring_input(0.01, 0.02),
        persona_votes: vec![
            PersonaVote {
                persona_id: "trend_breakout_fast".to_string(),
                stance: PersonaStance::Approve,
                conviction: 0.8,
                voice_power: 0.8,
                horizon: PersonaHorizon::Intraday,
                source_kind: EvidenceSourceKind::OfficialApiCollected,
                regime_fit: 0.9,
                data_quality_fit: 0.95,
                risk_fit: 0.8,
                expected_edge_fit: 0.7,
                doctrine_violations: vec![],
                reason_codes: vec![ReasonCode::PersonaVoteBuilt],
            },
            PersonaVote {
                persona_id: "defensive_value_risk".to_string(),
                stance: PersonaStance::Approve,
                conviction: 0.7,
                voice_power: 0.6,
                horizon: PersonaHorizon::Swing,
                source_kind: EvidenceSourceKind::OfficialApiCollected,
                regime_fit: 0.9,
                data_quality_fit: 0.95,
                risk_fit: 0.8,
                expected_edge_fit: 0.7,
                doctrine_violations: vec![],
                reason_codes: vec![ReasonCode::PersonaVoteBuilt],
            },
        ],
        target_horizon: PersonaHorizon::Swing,
        source_kind: EvidenceSourceKind::OfficialApiCollected,
        regime: Regime::TrendUp,
        reason_codes: vec![],
    };
    ChairV0 {
        config: ChairCommitteeConfig::default(),
    }
    .evaluate(&input)
}

#[test]
fn risk_denied_overrides_committee_approval_and_keeps_paper_only_path() {
    let bridge = CommitteeRiskBridge::default();
    let outcome = bridge.evaluate(
        &market_snapshot(0.95),
        &risk_snapshot(0.95),
        &scoring_input(0.0001, 0.02),
        approved_record(),
    );
    assert_eq!(outcome.risk_decision.kind, RiskDecisionKind::Deny);
    assert_eq!(outcome.final_action, CommitteeFinalAction::FinalDenied);
    assert!(outcome.risk_decision.approved_order_plan.is_none());
}

#[test]
fn emergency_stop_and_cooldown_override_everything() {
    let bridge = CommitteeRiskBridge::default();
    let emergency = bridge.evaluate(
        &market_snapshot(0.95),
        &RiskSnapshot {
            daily_pnl_pct: -0.10,
            ..risk_snapshot(0.95)
        },
        &scoring_input(0.01, 0.02),
        approved_record(),
    );
    let cooldown = bridge.evaluate(
        &market_snapshot(0.95),
        &RiskSnapshot {
            consecutive_losses: 4,
            ..risk_snapshot(0.95)
        },
        &scoring_input(0.01, 0.02),
        approved_record(),
    );
    assert_eq!(
        emergency.risk_decision.kind,
        RiskDecisionKind::EmergencyStop
    );
    assert_eq!(emergency.final_action, CommitteeFinalAction::FinalDenied);
    assert_eq!(cooldown.risk_decision.kind, RiskDecisionKind::Cooldown);
    assert_eq!(cooldown.final_action, CommitteeFinalAction::FinalDenied);
}

#[test]
fn approved_path_never_creates_real_execution() {
    let bridge = CommitteeRiskBridge::default();
    let outcome = bridge.evaluate(
        &market_snapshot(0.95),
        &risk_snapshot(0.95),
        &scoring_input(0.02, 0.01),
        approved_record(),
    );
    let plan = outcome
        .risk_decision
        .approved_order_plan
        .as_ref()
        .expect("paper-only plan");
    assert!(matches!(
        outcome.final_action,
        CommitteeFinalAction::PaperApprove | CommitteeFinalAction::PaperReduceSize
    ));
    assert!(plan.paper_only);
}
