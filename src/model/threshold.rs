use serde::{Deserialize, Serialize};

use crate::backtest::{TripleBarrierOutcome, TripleBarrierResult};
use crate::core::ReasonCode;
use crate::eval::{DatasetFrame, DatasetRow, DatasetSplitKind, TradeMetrics};

use super::prediction::PredictionFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizeMetric {
    NetReturn,
    ProfitFactor,
    SurvivalScore,
    RiskAdjustedReturn,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThresholdSet {
    pub p_win_threshold: f64,
    pub p_stop_threshold: f64,
    pub confidence_threshold: f64,
    pub no_trade_threshold: f64,
    pub min_expected_return_threshold: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThresholdSearchConfig {
    pub p_win_thresholds: Vec<f64>,
    pub p_stop_thresholds: Vec<f64>,
    pub confidence_thresholds: Vec<f64>,
    pub no_trade_thresholds: Vec<f64>,
    pub min_expected_return_thresholds: Vec<f64>,
    pub max_drawdown_constraint: Option<f64>,
    pub min_sample_count: usize,
    pub optimize_metric: OptimizeMetric,
    pub validation_only: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThresholdCandidateResult {
    pub thresholds: ThresholdSet,
    pub metrics: TradeMetrics,
    pub accepted: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThresholdSearchReport {
    pub fold_id: usize,
    pub candidates: Vec<ThresholdCandidateResult>,
    pub best_candidate: Option<ThresholdCandidateResult>,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn search_thresholds(
    fold_id: usize,
    dataset_frame: &DatasetFrame,
    prediction_frame: &PredictionFrame,
    config: &ThresholdSearchConfig,
) -> ThresholdSearchReport {
    let mut reason_codes = Vec::new();
    let mut candidate_rows = dataset_frame
        .rows
        .iter()
        .filter(|row| {
            row.fold_id == Some(fold_id) && row.split_kind == DatasetSplitKind::Validation
        })
        .collect::<Vec<_>>();
    if candidate_rows.is_empty() {
        reason_codes.push(ReasonCode::ThresholdResearchOnly);
        candidate_rows = dataset_frame
            .rows
            .iter()
            .filter(|row| row.fold_id == Some(fold_id) && row.split_kind == DatasetSplitKind::Test)
            .collect::<Vec<_>>();
    }

    let mut candidates = Vec::new();
    for &p_win_threshold in &config.p_win_thresholds {
        for &p_stop_threshold in &config.p_stop_thresholds {
            for &confidence_threshold in &config.confidence_thresholds {
                for &no_trade_threshold in &config.no_trade_thresholds {
                    for &min_expected_return_threshold in &config.min_expected_return_thresholds {
                        let thresholds = ThresholdSet {
                            p_win_threshold,
                            p_stop_threshold,
                            confidence_threshold,
                            no_trade_threshold,
                            min_expected_return_threshold,
                        };
                        let matching_results = candidate_rows
                            .iter()
                            .filter_map(|row| {
                                let prediction = prediction_frame.find_by_row_id(&row.row_id)?;
                                if prediction.p_win < p_win_threshold
                                    || prediction.p_stop > p_stop_threshold
                                    || prediction.confidence < confidence_threshold
                                    || prediction.no_trade_probability > no_trade_threshold
                                    || prediction.expected_return < min_expected_return_threshold
                                {
                                    return None;
                                }
                                triple_barrier_like_result(row)
                            })
                            .collect::<Vec<_>>();
                        let metrics = trade_metrics_from_results(&matching_results);
                        let mut candidate_reason_codes = Vec::new();
                        let accepted = if matching_results.len() < config.min_sample_count {
                            candidate_reason_codes.push(ReasonCode::ThresholdInsufficientSamples);
                            false
                        } else if config
                            .max_drawdown_constraint
                            .map(|limit| metrics.max_drawdown_pct > limit)
                            .unwrap_or(false)
                        {
                            false
                        } else {
                            candidate_reason_codes.push(ReasonCode::ThresholdSelectedOnValidation);
                            true
                        };

                        candidates.push(ThresholdCandidateResult {
                            thresholds,
                            metrics,
                            accepted,
                            reason_codes: candidate_reason_codes,
                        });
                    }
                }
            }
        }
    }

    let best_candidate = candidates
        .iter()
        .filter(|candidate| candidate.accepted)
        .max_by(|left, right| {
            score_candidate(left, config.optimize_metric)
                .total_cmp(&score_candidate(right, config.optimize_metric))
        })
        .cloned();

    ThresholdSearchReport {
        fold_id,
        candidates,
        best_candidate,
        reason_codes,
    }
}

fn triple_barrier_like_result(row: &DatasetRow) -> Option<TripleBarrierResult> {
    Some(TripleBarrierResult {
        outcome: row.label_outcome?,
        first_hit: row.label_first_hit?,
        entry_index: 0,
        exit_index: row.label_bars_held.unwrap_or(0),
        entry_price: 1.0,
        exit_price: 1.0 + row.label_net_return_pct.unwrap_or(0.0),
        gross_return_pct: row.label_gross_return_pct.unwrap_or(0.0),
        net_return_pct: row.label_net_return_pct.unwrap_or(0.0),
        max_favorable_excursion_pct: 0.0,
        max_adverse_excursion_pct: 0.0,
        bars_held: row.label_bars_held.unwrap_or(0),
        reason_codes: row.reason_codes.clone(),
    })
}

fn trade_metrics_from_results(results: &[TripleBarrierResult]) -> TradeMetrics {
    let total_trades = results.len();
    let wins = results
        .iter()
        .filter(|result| result.outcome == TripleBarrierOutcome::Win)
        .count();
    let losses = results
        .iter()
        .filter(|result| result.outcome == TripleBarrierOutcome::Loss)
        .count();
    let neutrals = results
        .iter()
        .filter(|result| result.outcome == TripleBarrierOutcome::Neutral)
        .count();
    let gross_return_pct = results
        .iter()
        .map(|result| result.gross_return_pct)
        .sum::<f64>();
    let net_return_pct = results
        .iter()
        .map(|result| result.net_return_pct)
        .sum::<f64>();
    let win_sum = results
        .iter()
        .filter(|result| result.net_return_pct > 0.0)
        .map(|result| result.net_return_pct)
        .sum::<f64>();
    let loss_sum = results
        .iter()
        .filter(|result| result.net_return_pct < 0.0)
        .map(|result| result.net_return_pct)
        .sum::<f64>();

    let mut equity: f64 = 1.0;
    let mut peak: f64 = 1.0;
    let mut max_drawdown_pct: f64 = 0.0;
    for result in results {
        equity *= 1.0 + result.net_return_pct;
        peak = peak.max(equity);
        max_drawdown_pct = max_drawdown_pct.max((1.0 - equity / peak.max(1e-9)).max(0.0));
    }

    TradeMetrics {
        total_trades,
        wins,
        losses,
        neutrals,
        win_rate: if total_trades == 0 {
            0.0
        } else {
            wins as f64 / total_trades as f64
        },
        avg_win_pct: average(
            &results
                .iter()
                .filter(|result| result.net_return_pct > 0.0)
                .map(|result| result.net_return_pct)
                .collect::<Vec<_>>(),
        ),
        avg_loss_pct: average(
            &results
                .iter()
                .filter(|result| result.net_return_pct < 0.0)
                .map(|result| result.net_return_pct)
                .collect::<Vec<_>>(),
        ),
        gross_return_pct,
        net_return_pct,
        profit_factor: if loss_sum.abs() > 0.0 {
            Some(win_sum / loss_sum.abs())
        } else {
            None
        },
        max_drawdown_pct,
        avg_bars_held: average(
            &results
                .iter()
                .map(|result| result.bars_held as f64)
                .collect::<Vec<_>>(),
        ),
        reason_codes: Vec::new(),
    }
}

fn score_candidate(candidate: &ThresholdCandidateResult, optimize_metric: OptimizeMetric) -> f64 {
    match optimize_metric {
        OptimizeMetric::NetReturn => candidate.metrics.net_return_pct,
        OptimizeMetric::ProfitFactor => candidate.metrics.profit_factor.unwrap_or(0.0),
        OptimizeMetric::SurvivalScore => {
            candidate.metrics.net_return_pct - candidate.metrics.max_drawdown_pct
        }
        OptimizeMetric::RiskAdjustedReturn => {
            candidate.metrics.net_return_pct / candidate.metrics.max_drawdown_pct.max(0.01)
        }
    }
}

fn average(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}
