use crate::core::{FeatureVector, MarketSnapshot, Regime};

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn regime_bias(regime: Regime) -> f64 {
    match regime {
        Regime::TrendUp | Regime::RiskOn => 0.85,
        Regime::TrendDown | Regime::RiskOff => -0.85,
        Regime::Range => 0.05,
        Regime::HighVolatility => -0.55,
        Regime::Panic => -0.9,
        Regime::Unknown => -0.35,
    }
}

pub fn derive_features(snapshot: &MarketSnapshot) -> FeatureVector {
    let liquidity_score = clamp01(snapshot.trade_value / 1_000_000.0);
    let spread_penalty = clamp01(snapshot.spread_bps / 20.0);
    let volatility_score = clamp01(snapshot.volatility / 0.05);
    let quality = clamp01(snapshot.data_quality_score);
    let bias = regime_bias(snapshot.regime);
    let trend_strength = clamp01(((bias + 1.0) / 2.0) * (1.0 - volatility_score * 0.4));
    let breakout_score = clamp01(trend_strength * 0.6 + liquidity_score * 0.25 + quality * 0.15);
    let overheat_score = clamp01(volatility_score * 0.7 + spread_penalty * 0.3);
    let no_trade_bias = clamp01(
        (1.0 - quality) * 0.4
            + spread_penalty * 0.25
            + volatility_score * 0.25
            + if matches!(snapshot.regime, Regime::Unknown | Regime::Panic) {
                0.2
            } else {
                0.0
            },
    );

    FeatureVector {
        trend_strength,
        breakout_score,
        liquidity_score,
        spread_penalty,
        volatility_score,
        data_quality_score: quality,
        regime_bias: bias,
        overheat_score,
        no_trade_bias,
    }
}
