use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, Side};

use super::{Candle, CandleSeries, CostModel};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TripleBarrierConfig {
    pub take_profit_pct: f64,
    pub stop_loss_pct: f64,
    pub horizon_bars: usize,
    pub fee_bps: f64,
    pub slippage_bps: f64,
    pub side: Side,
    pub use_high_low_intrabar: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TripleBarrierOutcome {
    Win,
    Loss,
    Neutral,
    NoData,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BarrierHit {
    TakeProfit,
    StopLoss,
    TimeExpired,
    NoData,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TripleBarrierResult {
    pub outcome: TripleBarrierOutcome,
    pub first_hit: BarrierHit,
    pub entry_index: usize,
    pub exit_index: usize,
    pub entry_price: f64,
    pub exit_price: f64,
    pub gross_return_pct: f64,
    pub net_return_pct: f64,
    pub max_favorable_excursion_pct: f64,
    pub max_adverse_excursion_pct: f64,
    pub bars_held: usize,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn evaluate_triple_barrier(
    series: &CandleSeries,
    entry_index: usize,
    entry_price: f64,
    config: TripleBarrierConfig,
) -> TripleBarrierResult {
    let mut reason_codes = vec![ReasonCode::DeterministicPath, ReasonCode::BacktestReplay];
    let Some(_entry_candle) = series.candle(entry_index) else {
        reason_codes.push(ReasonCode::OutcomeNoData);
        return no_data_result(entry_index, entry_price, reason_codes);
    };
    let exit_limit = entry_index + config.horizon_bars;
    if config.horizon_bars == 0 || exit_limit >= series.len() {
        reason_codes.push(ReasonCode::OutcomeNoData);
        return no_data_result(entry_index, entry_price, reason_codes);
    }

    let take_profit = match config.side {
        Side::Long => entry_price * (1.0 + config.take_profit_pct.max(0.0)),
        Side::Short => entry_price * (1.0 - config.take_profit_pct.max(0.0)),
    };
    let stop_loss = match config.side {
        Side::Long => entry_price * (1.0 - config.stop_loss_pct.max(0.0)),
        Side::Short => entry_price * (1.0 + config.stop_loss_pct.max(0.0)),
    };
    let cost_model = CostModel {
        fee_bps: config.fee_bps,
        slippage_bps: config.slippage_bps,
        spread_bps: series
            .candle(entry_index)
            .and_then(|candle| candle.spread_bps),
        min_cost_bps: None,
    };

    let mut max_favorable_excursion_pct = 0.0;
    let mut max_adverse_excursion_pct = 0.0;

    for current_index in (entry_index + 1)..=exit_limit {
        let candle = &series.candles[current_index];
        update_excursions(
            candle,
            entry_price,
            config.side,
            &mut max_favorable_excursion_pct,
            &mut max_adverse_excursion_pct,
        );
        let (take_hit, stop_hit) = candle_hits(
            candle,
            take_profit,
            stop_loss,
            config.side,
            config.use_high_low_intrabar,
        );

        if take_hit && stop_hit {
            reason_codes.push(ReasonCode::ConservativeSameCandleLoss);
            reason_codes.push(ReasonCode::StopLossHit);
            return barrier_result(
                TripleBarrierOutcome::Loss,
                BarrierHit::StopLoss,
                entry_index,
                current_index,
                entry_price,
                stop_loss,
                current_index - entry_index,
                max_favorable_excursion_pct,
                max_adverse_excursion_pct,
                config.side,
                cost_model,
                reason_codes,
            );
        }
        if take_hit {
            reason_codes.push(ReasonCode::TakeProfitHit);
            return barrier_result(
                TripleBarrierOutcome::Win,
                BarrierHit::TakeProfit,
                entry_index,
                current_index,
                entry_price,
                take_profit,
                current_index - entry_index,
                max_favorable_excursion_pct,
                max_adverse_excursion_pct,
                config.side,
                cost_model,
                reason_codes,
            );
        }
        if stop_hit {
            reason_codes.push(ReasonCode::StopLossHit);
            return barrier_result(
                TripleBarrierOutcome::Loss,
                BarrierHit::StopLoss,
                entry_index,
                current_index,
                entry_price,
                stop_loss,
                current_index - entry_index,
                max_favorable_excursion_pct,
                max_adverse_excursion_pct,
                config.side,
                cost_model,
                reason_codes,
            );
        }
    }

    reason_codes.push(ReasonCode::TimeBarrierExpired);
    barrier_result(
        TripleBarrierOutcome::Neutral,
        BarrierHit::TimeExpired,
        entry_index,
        exit_limit,
        entry_price,
        series.candles[exit_limit].close,
        config.horizon_bars,
        max_favorable_excursion_pct,
        max_adverse_excursion_pct,
        config.side,
        cost_model,
        reason_codes,
    )
}

fn no_data_result(
    entry_index: usize,
    entry_price: f64,
    mut reason_codes: Vec<ReasonCode>,
) -> TripleBarrierResult {
    if !reason_codes.contains(&ReasonCode::OutcomeNoData) {
        reason_codes.push(ReasonCode::OutcomeNoData);
    }
    TripleBarrierResult {
        outcome: TripleBarrierOutcome::NoData,
        first_hit: BarrierHit::NoData,
        entry_index,
        exit_index: entry_index,
        entry_price,
        exit_price: entry_price,
        gross_return_pct: 0.0,
        net_return_pct: 0.0,
        max_favorable_excursion_pct: 0.0,
        max_adverse_excursion_pct: 0.0,
        bars_held: 0,
        reason_codes,
    }
}

fn barrier_result(
    outcome: TripleBarrierOutcome,
    first_hit: BarrierHit,
    entry_index: usize,
    exit_index: usize,
    entry_price: f64,
    exit_price: f64,
    bars_held: usize,
    max_favorable_excursion_pct: f64,
    max_adverse_excursion_pct: f64,
    side: Side,
    cost_model: CostModel,
    mut reason_codes: Vec<ReasonCode>,
) -> TripleBarrierResult {
    let gross_return_pct = gross_return_pct(entry_price, exit_price, side, first_hit);
    let net_return_pct = cost_model.net_return_after_cost(gross_return_pct);
    reason_codes.push(ReasonCode::CostApplied);
    TripleBarrierResult {
        outcome,
        first_hit,
        entry_index,
        exit_index,
        entry_price,
        exit_price,
        gross_return_pct,
        net_return_pct,
        max_favorable_excursion_pct,
        max_adverse_excursion_pct,
        bars_held,
        reason_codes,
    }
}

fn gross_return_pct(entry_price: f64, exit_price: f64, side: Side, first_hit: BarrierHit) -> f64 {
    if matches!(first_hit, BarrierHit::NoData) || exit_price <= 0.0 || entry_price <= 0.0 {
        0.0
    } else {
        match side {
            Side::Long => (exit_price / entry_price) - 1.0,
            Side::Short => (entry_price / exit_price) - 1.0,
        }
    }
}

fn candle_hits(
    candle: &Candle,
    take_profit: f64,
    stop_loss: f64,
    side: Side,
    use_high_low_intrabar: bool,
) -> (bool, bool) {
    match (side, use_high_low_intrabar) {
        (Side::Long, true) => (candle.high >= take_profit, candle.low <= stop_loss),
        (Side::Long, false) => (candle.close >= take_profit, candle.close <= stop_loss),
        (Side::Short, true) => (candle.low <= take_profit, candle.high >= stop_loss),
        (Side::Short, false) => (candle.close <= take_profit, candle.close >= stop_loss),
    }
}

fn update_excursions(
    candle: &Candle,
    entry_price: f64,
    side: Side,
    max_favorable_excursion_pct: &mut f64,
    max_adverse_excursion_pct: &mut f64,
) {
    let (favorable, adverse) = match side {
        Side::Long => (
            ((candle.high / entry_price.max(1e-9)) - 1.0).max(0.0),
            (1.0 - candle.low / entry_price.max(1e-9)).max(0.0),
        ),
        Side::Short => (
            (1.0 - candle.low / entry_price.max(1e-9)).max(0.0),
            ((candle.high / entry_price.max(1e-9)) - 1.0).max(0.0),
        ),
    };
    *max_favorable_excursion_pct = max_favorable_excursion_pct.max(favorable);
    *max_adverse_excursion_pct = max_adverse_excursion_pct.max(adverse);
}
