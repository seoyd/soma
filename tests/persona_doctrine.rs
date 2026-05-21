use soma_zero::core::{MarketSnapshot, ReasonCode, Regime, SignalOutput, Stance};
use soma_zero::league::{
    CycleRiskSkeptic, DoctrineObservation, MomentumTrendFast, Persona, ValueQualityFilter,
    active_persona_cards,
};

fn market() -> MarketSnapshot {
    MarketSnapshot {
        symbol: "AAPL".to_string(),
        timestamp_ms: 1_715_000_000_000,
        price: 100.0,
        bid: 99.9,
        ask: 100.1,
        spread_bps: 4.0,
        volume: 20_000.0,
        trade_value: 2_000_000.0,
        volatility: 0.01,
        regime: Regime::TrendUp,
        data_quality_score: 0.95,
    }
}

fn signal() -> SignalOutput {
    SignalOutput {
        symbol: "AAPL".to_string(),
        horizon_bars: 48,
        p_win: 0.62,
        p_stop: 0.32,
        expected_return: 0.012,
        expected_drawdown: 0.006,
        confidence: 0.70,
        no_trade_probability: 0.22,
        source: "test".to_string(),
    }
}

#[test]
fn persona_doctrine_cards_construct_correctly() {
    let cards = active_persona_cards();
    assert_eq!(cards.len(), 3);
    assert_eq!(cards[0].persona_id, "momentum_trend_fast");
    assert_eq!(cards[1].persona_id, "value_quality_filter");
    assert_eq!(cards[2].persona_id, "cycle_risk_skeptic");
}

#[test]
fn momentum_trend_fast_rejects_averaging_down_doctrine_violation() {
    let persona = MomentumTrendFast::default();
    let check = persona.doctrine_check(&DoctrineObservation {
        stance: Stance::Buy,
        is_adding_to_loser: true,
        ..DoctrineObservation::default()
    });

    assert!(!check.violations.is_empty());
    assert!(
        check
            .reason_codes
            .contains(&ReasonCode::AveragingDownRejected)
    );
    assert!(check.reason_codes.contains(&ReasonCode::DoctrineViolation));
}

#[test]
fn value_quality_filter_does_not_emit_intraday_entry_signal() {
    let persona = ValueQualityFilter::default();
    let mut fast_signal = signal();
    fast_signal.horizon_bars = 4;

    let vote = persona.vote(&market(), &fast_signal);
    assert_eq!(vote.stance, Stance::NoTrade);
    assert!(
        vote.reason_codes
            .contains(&ReasonCode::IntradayEntryForbidden)
    );
}

#[test]
fn cycle_risk_skeptic_can_emit_veto_warning() {
    let persona = CycleRiskSkeptic::default();
    let mut stressed_market = market();
    stressed_market.regime = Regime::Panic;
    stressed_market.volatility = 0.06;

    let vote = persona.vote(&stressed_market, &signal());
    assert_eq!(vote.stance, Stance::NoTrade);
    assert!(vote.veto);
    assert!(vote.reason_codes.contains(&ReasonCode::CycleSkepticVeto));
    assert!(vote.reason_codes.contains(&ReasonCode::OverheatedMarket));
}
