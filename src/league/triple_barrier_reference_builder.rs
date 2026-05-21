use serde::{Deserialize, Serialize};

use crate::CandleSeries;
use crate::backtest::CostModel;
use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::EvidenceSourceKind;

use super::candle_alignment::{CandleAlignmentRecord, CandleAlignmentStatus};
use super::committee_outcome_reference::{CommitteeOutcomeReference, CommitteeTripleBarrierLabel};
use super::committee_scenario_loader::CommitteeScenarioRow;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TripleBarrierTieBreakPolicy {
    #[default]
    StopFirst,
    TakeProfitFirst,
    TimeOrder,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TripleBarrierReferenceSource {
    #[default]
    LocalCandleSeries,
    EstimatedDiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TripleBarrierReferenceConfig {
    pub horizon_bars: usize,
    pub take_profit_pct: f64,
    pub stop_loss_pct: f64,
    pub cost_bps: f64,
    pub slippage_bps: f64,
    #[serde(default)]
    pub tie_break_policy: TripleBarrierTieBreakPolicy,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TripleBarrierReferenceBuildResult {
    pub reference: CommitteeOutcomeReference,
    pub generated_from: TripleBarrierReferenceSource,
    pub diagnostic_only: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TripleBarrierReferenceBuilder;

impl Default for TripleBarrierReferenceConfig {
    fn default() -> Self {
        Self {
            horizon_bars: 24,
            take_profit_pct: 0.02,
            stop_loss_pct: 0.01,
            cost_bps: 5.0,
            slippage_bps: 2.0,
            tie_break_policy: TripleBarrierTieBreakPolicy::StopFirst,
            reason_codes: vec![ReasonCode::CommitteeOutcomeReferenceBuilt],
        }
    }
}

impl TripleBarrierReferenceBuilder {
    pub fn build(
        &self,
        row: &CommitteeScenarioRow,
        alignment: &CandleAlignmentRecord,
        series: &CandleSeries,
        config: &TripleBarrierReferenceConfig,
        diagnostic_only: bool,
    ) -> Result<TripleBarrierReferenceBuildResult, String> {
        if !matches!(
            alignment.status,
            CandleAlignmentStatus::MatchedExact | CandleAlignmentStatus::MatchedWithTolerance
        ) {
            return Err("triple barrier reference requires matched candle alignment".to_string());
        }
        let entry_index = alignment
            .matched_start_index
            .ok_or_else(|| "triple barrier reference missing entry index".to_string())?;
        let future_start = alignment
            .future_window_start_index
            .ok_or_else(|| "triple barrier reference missing future window start".to_string())?;
        let future_end = alignment
            .future_window_end_index
            .ok_or_else(|| "triple barrier reference missing future window end".to_string())?;
        if future_end >= series.len() || future_start <= entry_index || future_start > future_end {
            return Err("triple barrier reference future window is invalid".to_string());
        }
        let entry_price = series
            .candle(entry_index)
            .map(|candle| candle.close)
            .ok_or_else(|| "triple barrier reference missing entry candle".to_string())?;
        let take_price = entry_price * (1.0 + config.take_profit_pct.max(0.0));
        let stop_price = entry_price * (1.0 - config.stop_loss_pct.max(0.0));
        let cost_model = CostModel {
            fee_bps: config.cost_bps,
            slippage_bps: config.slippage_bps,
            spread_bps: series
                .candle(entry_index)
                .and_then(|candle| candle.spread_bps),
            min_cost_bps: None,
        };
        let mut label = CommitteeTripleBarrierLabel::TimeExpired;
        let mut exit_price = series
            .candle(future_end)
            .map(|candle| candle.close)
            .unwrap_or(entry_price);
        let mut reason_codes = vec![
            ReasonCode::CommitteeOutcomeReferenceBuilt,
            ReasonCode::DeterministicPath,
        ];
        let mut max_favorable_excursion_pct: f64 = 0.0;
        let mut max_adverse_excursion_pct: f64 = 0.0;
        for current_index in future_start..=future_end {
            let candle = series.candle(current_index).ok_or_else(|| {
                "triple barrier reference candle window out of bounds".to_string()
            })?;
            max_favorable_excursion_pct =
                max_favorable_excursion_pct.max((candle.high / entry_price.max(1e-9)) - 1.0);
            max_adverse_excursion_pct =
                max_adverse_excursion_pct.min((candle.low / entry_price.max(1e-9)) - 1.0);
            let take_hit = candle.high >= take_price;
            let stop_hit = candle.low <= stop_price;
            if take_hit || stop_hit {
                let chosen = choose_label(
                    candle,
                    entry_price,
                    take_price,
                    stop_price,
                    config.tie_break_policy,
                    take_hit,
                    stop_hit,
                );
                label = chosen;
                exit_price = match chosen {
                    CommitteeTripleBarrierLabel::TakeProfit => take_price,
                    CommitteeTripleBarrierLabel::StopLoss => stop_price,
                    CommitteeTripleBarrierLabel::TimeExpired
                    | CommitteeTripleBarrierLabel::NoTradeCounterfactual
                    | CommitteeTripleBarrierLabel::RiskDeniedCounterfactual
                    | CommitteeTripleBarrierLabel::Unknown => candle.close,
                };
                match label {
                    CommitteeTripleBarrierLabel::TakeProfit => {
                        reason_codes.push(ReasonCode::TakeProfitHit)
                    }
                    CommitteeTripleBarrierLabel::StopLoss => {
                        reason_codes.push(ReasonCode::StopLossHit)
                    }
                    _ => {}
                }
                if take_hit && stop_hit && label == CommitteeTripleBarrierLabel::StopLoss {
                    reason_codes.push(ReasonCode::ConservativeSameCandleLoss);
                }
                break;
            }
        }
        if label == CommitteeTripleBarrierLabel::TimeExpired {
            reason_codes.push(ReasonCode::TimeBarrierExpired);
        }
        let gross_return_pct = match label {
            CommitteeTripleBarrierLabel::TakeProfit
            | CommitteeTripleBarrierLabel::TimeExpired
            | CommitteeTripleBarrierLabel::NoTradeCounterfactual
            | CommitteeTripleBarrierLabel::RiskDeniedCounterfactual => {
                (exit_price / entry_price.max(1e-9)) - 1.0
            }
            CommitteeTripleBarrierLabel::StopLoss => (exit_price / entry_price.max(1e-9)) - 1.0,
            CommitteeTripleBarrierLabel::Unknown => 0.0,
        };
        let net_return_pct = cost_model.net_return_after_cost(gross_return_pct);
        reason_codes.push(ReasonCode::CostApplied);
        let no_lookahead_safe = alignment.no_lookahead_safe
            && matches!(
                alignment.status,
                CandleAlignmentStatus::MatchedExact | CandleAlignmentStatus::MatchedWithTolerance
            );
        let generated_from = if diagnostic_only {
            TripleBarrierReferenceSource::EstimatedDiagnosticOnly
        } else {
            TripleBarrierReferenceSource::LocalCandleSeries
        };
        Ok(TripleBarrierReferenceBuildResult {
            generated_from,
            diagnostic_only,
            reference: CommitteeOutcomeReference {
                outcome_id: format!("{}-outcome", row.scenario_row_id),
                decision_id: Some(row.scenario_row_id.clone()),
                symbol: row.symbol.clone(),
                timestamp_ms: row.timestamp_ms,
                horizon_bars: config.horizon_bars,
                triple_barrier_label: label,
                net_return_pct: Some(net_return_pct),
                max_favorable_excursion_pct: Some(max_favorable_excursion_pct.max(0.0)),
                max_adverse_excursion_pct: Some(max_adverse_excursion_pct.min(0.0)),
                cost_bps: config.cost_bps,
                slippage_bps: config.slippage_bps,
                source_kind: source_kind_for_row(row, diagnostic_only),
                no_lookahead_safe,
                reason_codes: stable_reason_codes(
                    &config
                        .reason_codes
                        .iter()
                        .cloned()
                        .chain(reason_codes)
                        .collect::<Vec<_>>(),
                ),
            },
        })
    }
}

fn choose_label(
    candle: &crate::Candle,
    entry_price: f64,
    take_price: f64,
    stop_price: f64,
    tie_break_policy: TripleBarrierTieBreakPolicy,
    take_hit: bool,
    stop_hit: bool,
) -> CommitteeTripleBarrierLabel {
    match (take_hit, stop_hit) {
        (true, false) => CommitteeTripleBarrierLabel::TakeProfit,
        (false, true) => CommitteeTripleBarrierLabel::StopLoss,
        (true, true) => match tie_break_policy {
            TripleBarrierTieBreakPolicy::StopFirst => CommitteeTripleBarrierLabel::StopLoss,
            TripleBarrierTieBreakPolicy::TakeProfitFirst => CommitteeTripleBarrierLabel::TakeProfit,
            TripleBarrierTieBreakPolicy::TimeOrder => {
                let take_distance = (take_price - candle.open).abs();
                let stop_distance = (candle.open - stop_price).abs();
                if take_distance < stop_distance {
                    CommitteeTripleBarrierLabel::TakeProfit
                } else if stop_distance < take_distance {
                    CommitteeTripleBarrierLabel::StopLoss
                } else if candle.close >= entry_price {
                    CommitteeTripleBarrierLabel::TakeProfit
                } else {
                    CommitteeTripleBarrierLabel::StopLoss
                }
            }
        },
        (false, false) => CommitteeTripleBarrierLabel::TimeExpired,
    }
}

fn source_kind_for_row(row: &CommitteeScenarioRow, diagnostic_only: bool) -> EvidenceSourceKind {
    if diagnostic_only {
        EvidenceSourceKind::GeneratedSynthetic
    } else {
        row.evidence_source_kind
    }
}
