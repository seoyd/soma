use soma_zero::{
    CommitteeRiskBridge, EvidenceSourceKind, MarketSnapshot, PersonaHorizon, PersonaScoringInput,
    ProviderMarket, ReasonCode, Regime, RiskDecisionKind, RiskSnapshot, SignalOutput,
    build_risk_bridge_diagnostics,
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

fn scoring_input(expected_edge_after_cost: f64, data_quality_score: f64) -> PersonaScoringInput {
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
            expected_drawdown: 0.02,
            confidence: 0.82,
            no_trade_probability: 0.18,
            source: "test".to_string(),
        },
        data_quality_score,
        spread_bps: Some(8.0),
        expected_edge_after_cost,
        expected_drawdown: 0.02,
        risk_snapshot: Some(RiskSnapshot {
            daily_pnl_pct: 0.0,
            consecutive_losses: 0,
            current_positions_count: 0,
            total_exposure_pct: 0.0,
            symbol_exposure_pct: 0.0,
            api_health_score: 1.0,
            data_quality_score,
        }),
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

#[test]
fn denial_maps_to_final_denied_and_flags_data_quality() {
    let bridge = CommitteeRiskBridge::default();
    let input = scoring_input(0.02, 0.40);
    let record = soma_zero::ChairV0::default().evaluate(&soma_zero::CommitteeInput {
        scoring_input: input.clone(),
        persona_votes: vec![soma_zero::PersonaVote {
            persona_id: "trend_breakout_fast".to_string(),
            stance: soma_zero::PersonaStance::Approve,
            conviction: 0.8,
            voice_power: 0.8,
            horizon: PersonaHorizon::Intraday,
            source_kind: EvidenceSourceKind::OfficialApiCollected,
            regime_fit: 0.9,
            data_quality_fit: 0.4,
            risk_fit: 0.8,
            expected_edge_fit: 0.8,
            doctrine_violations: vec![],
            reason_codes: vec![ReasonCode::PersonaVoteBuilt],
        }],
        target_horizon: PersonaHorizon::Swing,
        source_kind: EvidenceSourceKind::OfficialApiCollected,
        regime: Regime::TrendUp,
        reason_codes: vec![],
    });
    let outcome = bridge.evaluate(
        &market_snapshot(0.40),
        &RiskSnapshot {
            data_quality_score: 0.40,
            ..input.risk_snapshot.clone().expect("risk")
        },
        &input,
        record.clone(),
    );
    let diagnostics =
        build_risk_bridge_diagnostics(&bridge, &market_snapshot(0.40), &input, &record, &outcome);
    assert!(diagnostics.veto_applied);
    assert!(diagnostics.data_quality_block);
}

#[test]
fn emergency_and_cooldown_are_reported() {
    let bridge = CommitteeRiskBridge::default();
    let input = scoring_input(0.02, 0.95);
    let record = soma_zero::ChairV0::default().evaluate(&soma_zero::CommitteeInput {
        scoring_input: input.clone(),
        persona_votes: vec![soma_zero::PersonaVote {
            persona_id: "trend_breakout_fast".to_string(),
            stance: soma_zero::PersonaStance::Approve,
            conviction: 0.8,
            voice_power: 0.8,
            horizon: PersonaHorizon::Intraday,
            source_kind: EvidenceSourceKind::OfficialApiCollected,
            regime_fit: 0.9,
            data_quality_fit: 0.9,
            risk_fit: 0.8,
            expected_edge_fit: 0.8,
            doctrine_violations: vec![],
            reason_codes: vec![ReasonCode::PersonaVoteBuilt],
        }],
        target_horizon: PersonaHorizon::Swing,
        source_kind: EvidenceSourceKind::OfficialApiCollected,
        regime: Regime::TrendUp,
        reason_codes: vec![],
    });
    let emergency = bridge.evaluate(
        &market_snapshot(0.95),
        &RiskSnapshot {
            daily_pnl_pct: -0.10,
            ..input.risk_snapshot.clone().expect("risk")
        },
        &input,
        record.clone(),
    );
    let cooldown = bridge.evaluate(
        &market_snapshot(0.95),
        &RiskSnapshot {
            consecutive_losses: 5,
            ..input.risk_snapshot.clone().expect("risk")
        },
        &input,
        record.clone(),
    );
    assert_eq!(
        emergency.risk_decision.kind,
        RiskDecisionKind::EmergencyStop
    );
    assert_eq!(cooldown.risk_decision.kind, RiskDecisionKind::Cooldown);
}
