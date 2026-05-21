use soma_zero::{
    CycleRegimeGuardScorer, DefensiveValueRiskScorer, EvidenceSourceKind, FeatureName,
    FeatureValue, FeatureVector, PersonaHorizon, PersonaScorer, PersonaScoringInput, PersonaStance,
    ProviderMarket, ReasonCode, Regime, RiskSnapshot, SignalOutput, Timeframe,
    TrendBreakoutFastScorer,
};

fn scoring_input(
    regime: Regime,
    expected_edge_after_cost: f64,
    expected_drawdown: f64,
    data_quality_score: f64,
    confidence: f64,
    spread_bps: f64,
    breakout_score: f64,
    volume_z20: f64,
) -> PersonaScoringInput {
    PersonaScoringInput {
        symbol: "BTC-KRW".to_string(),
        timestamp_ms: 1_700_000_000_000,
        source_kind: EvidenceSourceKind::OfficialApiCollected,
        market: ProviderMarket::Crypto,
        target_horizon: PersonaHorizon::Swing,
        feature_vector: Some(FeatureVector {
            symbol: "BTC-KRW".to_string(),
            timestamp_ms: 1_700_000_000_000,
            timeframe: Timeframe::OneMinute,
            feature_names: vec![FeatureName::CloseOverMa20, FeatureName::VolumeZ20],
            values: vec![
                FeatureValue::Value(breakout_score),
                FeatureValue::Value(volume_z20),
            ],
            data_quality_score,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }),
        regime,
        signal_output: SignalOutput {
            symbol: "BTC-KRW".to_string(),
            horizon_bars: 12,
            p_win: 0.62,
            p_stop: 0.28,
            expected_return: expected_edge_after_cost,
            expected_drawdown,
            confidence,
            no_trade_probability: (1.0 - confidence).clamp(0.0, 1.0),
            source: "test".to_string(),
        },
        data_quality_score,
        spread_bps: Some(spread_bps),
        expected_edge_after_cost,
        expected_drawdown,
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
fn trend_persona_approves_strong_trend_volume_fixture() {
    let vote = TrendBreakoutFastScorer.score(&scoring_input(
        Regime::TrendUp,
        0.015,
        0.02,
        0.92,
        0.82,
        4.0,
        0.82,
        1.8,
    ));
    assert!(matches!(
        vote.stance,
        PersonaStance::Approve | PersonaStance::StrongApprove
    ));
}

#[test]
fn trend_persona_no_trades_weak_breakout() {
    let vote = TrendBreakoutFastScorer.score(&scoring_input(
        Regime::Range,
        0.004,
        0.02,
        0.92,
        0.40,
        4.0,
        0.42,
        0.1,
    ));
    assert_eq!(vote.stance, PersonaStance::NoTrade);
}

#[test]
fn defensive_persona_vetoes_negative_edge_and_poor_quality() {
    let negative_edge = DefensiveValueRiskScorer.score(&scoring_input(
        Regime::Range,
        -0.002,
        0.02,
        0.90,
        0.70,
        4.0,
        0.60,
        1.0,
    ));
    let poor_quality = DefensiveValueRiskScorer.score(&scoring_input(
        Regime::Range,
        0.006,
        0.02,
        0.60,
        0.70,
        4.0,
        0.60,
        1.0,
    ));
    assert_eq!(negative_edge.stance, PersonaStance::Veto);
    assert!(matches!(
        poor_quality.stance,
        PersonaStance::NoTrade | PersonaStance::Veto
    ));
}

#[test]
fn regime_persona_reduces_size_under_high_volatility_and_vetoes_panic() {
    let reduce = CycleRegimeGuardScorer.score(&scoring_input(
        Regime::TrendUp,
        0.010,
        0.08,
        0.90,
        0.75,
        6.0,
        0.75,
        1.2,
    ));
    let veto = CycleRegimeGuardScorer.score(&scoring_input(
        Regime::Panic,
        0.010,
        0.03,
        0.50,
        0.75,
        6.0,
        0.75,
        1.2,
    ));
    assert_eq!(reduce.stance, PersonaStance::ReduceSize);
    assert_eq!(veto.stance, PersonaStance::Veto);
}

#[test]
fn convictions_and_voice_power_are_bounded() {
    for vote in [
        TrendBreakoutFastScorer.score(&scoring_input(
            Regime::TrendUp,
            0.05,
            0.0,
            1.0,
            1.0,
            0.0,
            1.0,
            4.0,
        )),
        DefensiveValueRiskScorer.score(&scoring_input(
            Regime::Range,
            0.03,
            0.01,
            1.0,
            1.0,
            0.0,
            1.0,
            4.0,
        )),
        CycleRegimeGuardScorer.score(&scoring_input(
            Regime::RiskOn,
            0.03,
            0.01,
            1.0,
            1.0,
            0.0,
            1.0,
            4.0,
        )),
    ] {
        assert!((0.0..=1.0).contains(&vote.conviction));
        assert!((0.0..=1.0).contains(&vote.voice_power));
    }
}
