use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::backtest::{DecisionRecord, OutcomeRecord, TripleBarrierOutcome};
use crate::core::{ChairDecisionKind, ReasonCode, Regime, RiskDecisionKind};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TradeMetrics {
    pub total_trades: usize,
    pub wins: usize,
    pub losses: usize,
    pub neutrals: usize,
    pub win_rate: f64,
    pub avg_win_pct: f64,
    pub avg_loss_pct: f64,
    pub gross_return_pct: f64,
    pub net_return_pct: f64,
    pub profit_factor: Option<f64>,
    pub max_drawdown_pct: f64,
    pub avg_bars_held: f64,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DecisionMetrics {
    pub total_decisions: usize,
    pub executed: usize,
    pub denied_by_risk: usize,
    pub no_trade: usize,
    pub require_confirm_count: usize,
    pub approve_candidate_count: usize,
    pub reason_code_counts: BTreeMap<String, usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NoTradeMetrics {
    pub no_trade_count: usize,
    pub avoided_loss_count: usize,
    pub missed_gain_count: usize,
    pub avg_avoided_loss_score: f64,
    pub avg_missed_gain_penalty: f64,
    pub net_silence_value: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiskGovernorMetrics {
    pub denied_count: usize,
    pub emergency_stop_count: usize,
    pub cooldown_count: usize,
    pub avoided_loss_count: usize,
    pub missed_gain_count: usize,
    pub defensive_value: f64,
    pub opportunity_cost: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CalibrationBin {
    pub bin_lower: f64,
    pub bin_upper: f64,
    pub count: usize,
    pub predicted_avg: f64,
    pub actual_win_rate: f64,
    pub brier_score: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CalibrationMetrics {
    pub brier_score: f64,
    pub calibration_bins: Vec<CalibrationBin>,
    pub expected_calibration_error: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegimeMetrics {
    pub regime: Regime,
    pub trade_metrics: TradeMetrics,
    pub decision_metrics: DecisionMetrics,
    pub no_trade_metrics: NoTradeMetrics,
    pub risk_metrics: RiskGovernorMetrics,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonaFoldMetrics {
    pub persona_id: String,
    pub selected_count: usize,
    pub shadow_count: usize,
    pub supported_final_count: usize,
    pub opposed_final_count: usize,
    pub forced_contrarian_count: usize,
    pub avg_contribution_score: f64,
    pub net_attributed_return_pct: f64,
    pub high_confidence_miss_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChairMetrics {
    pub approve_candidate_count: usize,
    pub reduce_size_count: usize,
    pub no_trade_count: usize,
    pub require_confirm_count: usize,
    pub groupthink_risk_avg: f64,
    pub disagreement_score_avg: f64,
    pub cluster_penalty_avg: f64,
}

pub fn compute_trade_metrics(outcomes: &[OutcomeRecord]) -> TradeMetrics {
    let results = outcomes
        .iter()
        .filter_map(|record| record.triple_barrier_result.as_ref())
        .filter(|result| result.outcome != TripleBarrierOutcome::NoData)
        .collect::<Vec<_>>();
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
    let gross_return_pct = results
        .iter()
        .map(|result| result.gross_return_pct)
        .sum::<f64>();
    let net_return_pct = results
        .iter()
        .map(|result| result.net_return_pct)
        .sum::<f64>();
    let avg_bars_held = average(
        &results
            .iter()
            .map(|result| result.bars_held as f64)
            .collect::<Vec<_>>(),
    );

    let mut equity: f64 = 1.0;
    let mut peak: f64 = 1.0;
    let mut max_drawdown_pct: f64 = 0.0;
    for result in &results {
        equity *= 1.0 + result.net_return_pct;
        peak = peak.max(equity);
        max_drawdown_pct = max_drawdown_pct.max((1.0 - equity / peak.max(1e-9)).max(0.0));
    }

    TradeMetrics {
        total_trades,
        wins,
        losses,
        neutrals,
        win_rate: safe_ratio(wins as f64, total_trades as f64),
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
        avg_bars_held,
        reason_codes: vec![ReasonCode::DeterministicPath, ReasonCode::CostApplied],
    }
}

pub fn compute_decision_metrics(
    decisions: &[DecisionRecord],
    outcomes: &[OutcomeRecord],
) -> DecisionMetrics {
    let mut reason_code_counts = BTreeMap::new();
    for decision in decisions {
        for reason in &decision.reason_codes {
            *reason_code_counts.entry(format!("{reason:?}")).or_insert(0) += 1;
        }
    }

    DecisionMetrics {
        total_decisions: decisions.len(),
        executed: decisions
            .iter()
            .filter(|decision| decision.selected_for_execution)
            .count(),
        denied_by_risk: outcomes
            .iter()
            .filter(|outcome| outcome.denied_by_risk)
            .count(),
        no_trade: outcomes.iter().filter(|outcome| outcome.no_trade).count(),
        require_confirm_count: decisions
            .iter()
            .filter(|decision| decision.chair_output.decision == ChairDecisionKind::RequireConfirm)
            .count(),
        approve_candidate_count: decisions
            .iter()
            .filter(|decision| {
                decision.chair_output.decision == ChairDecisionKind::ApproveCandidate
            })
            .count(),
        reason_code_counts,
    }
}

pub fn compute_no_trade_metrics(outcomes: &[OutcomeRecord]) -> NoTradeMetrics {
    let no_trade_outcomes = outcomes
        .iter()
        .filter(|outcome| outcome.no_trade)
        .collect::<Vec<_>>();
    let avoided = no_trade_outcomes
        .iter()
        .filter(|outcome| outcome.avoided_loss_score > 0.0)
        .collect::<Vec<_>>();
    let missed = no_trade_outcomes
        .iter()
        .filter(|outcome| outcome.missed_gain_penalty < 0.0)
        .collect::<Vec<_>>();

    NoTradeMetrics {
        no_trade_count: no_trade_outcomes.len(),
        avoided_loss_count: avoided.len(),
        missed_gain_count: missed.len(),
        avg_avoided_loss_score: average(
            &avoided
                .iter()
                .map(|outcome| outcome.avoided_loss_score)
                .collect::<Vec<_>>(),
        ),
        avg_missed_gain_penalty: average(
            &missed
                .iter()
                .map(|outcome| outcome.missed_gain_penalty)
                .collect::<Vec<_>>(),
        ),
        net_silence_value: no_trade_outcomes
            .iter()
            .map(|outcome| outcome.avoided_loss_score + outcome.missed_gain_penalty)
            .sum(),
    }
}

pub fn compute_risk_metrics(
    decisions: &[DecisionRecord],
    outcomes: &[OutcomeRecord],
) -> RiskGovernorMetrics {
    let denied_outcomes = outcomes
        .iter()
        .filter(|outcome| outcome.denied_by_risk)
        .collect::<Vec<_>>();
    RiskGovernorMetrics {
        denied_count: denied_outcomes.len(),
        emergency_stop_count: decisions
            .iter()
            .filter(|decision| decision.risk_decision.kind == RiskDecisionKind::EmergencyStop)
            .count(),
        cooldown_count: decisions
            .iter()
            .filter(|decision| decision.risk_decision.kind == RiskDecisionKind::Cooldown)
            .count(),
        avoided_loss_count: denied_outcomes
            .iter()
            .filter(|outcome| outcome.avoided_loss_score > 0.0)
            .count(),
        missed_gain_count: denied_outcomes
            .iter()
            .filter(|outcome| outcome.missed_gain_penalty < 0.0)
            .count(),
        defensive_value: denied_outcomes
            .iter()
            .map(|outcome| outcome.avoided_loss_score)
            .sum(),
        opportunity_cost: denied_outcomes
            .iter()
            .map(|outcome| outcome.missed_gain_penalty.abs())
            .sum(),
    }
}

pub fn compute_calibration_metrics(
    decisions: &[DecisionRecord],
    outcomes: &[OutcomeRecord],
) -> CalibrationMetrics {
    let outcome_map = outcomes
        .iter()
        .map(|outcome| (outcome.decision_id.clone(), outcome))
        .collect::<BTreeMap<_, _>>();
    let observations = decisions
        .iter()
        .filter_map(|decision| {
            let outcome = outcome_map.get(&decision.id)?;
            let actual = if outcome
                .triple_barrier_result
                .as_ref()
                .map(|result| result.outcome == TripleBarrierOutcome::Win)
                .unwrap_or(false)
            {
                1.0
            } else {
                0.0
            };
            Some((decision.signal_output.p_win.clamp(0.0, 1.0), actual))
        })
        .collect::<Vec<_>>();
    let brier_score = average(
        &observations
            .iter()
            .map(|(predicted, actual)| {
                let diff = predicted - actual;
                diff * diff
            })
            .collect::<Vec<_>>(),
    );

    let bins = [
        (0.0, 0.2),
        (0.2, 0.4),
        (0.4, 0.6),
        (0.6, 0.8),
        (0.8, 1.000_000_1),
    ];
    let mut calibration_bins = Vec::new();
    let mut ece = 0.0;
    for (lower, upper) in bins {
        let bucket = observations
            .iter()
            .filter(|(predicted, _)| *predicted >= lower && *predicted < upper)
            .collect::<Vec<_>>();
        let count = bucket.len();
        let predicted_avg = average(
            &bucket
                .iter()
                .map(|(predicted, _)| *predicted)
                .collect::<Vec<_>>(),
        );
        let actual_win_rate =
            average(&bucket.iter().map(|(_, actual)| *actual).collect::<Vec<_>>());
        if !observations.is_empty() {
            ece += ((count as f64) / (observations.len() as f64))
                * (predicted_avg - actual_win_rate).abs();
        }
        calibration_bins.push(CalibrationBin {
            bin_lower: lower,
            bin_upper: upper.min(1.0),
            count,
            predicted_avg,
            actual_win_rate,
            brier_score: average(
                &bucket
                    .iter()
                    .map(|(predicted, actual)| {
                        let diff = predicted - actual;
                        diff * diff
                    })
                    .collect::<Vec<_>>(),
            ),
        });
    }

    CalibrationMetrics {
        brier_score,
        calibration_bins,
        expected_calibration_error: if observations.is_empty() {
            None
        } else {
            Some(ece)
        },
    }
}

pub fn compute_regime_metrics(
    decisions: &[DecisionRecord],
    outcomes: &[OutcomeRecord],
) -> Vec<RegimeMetrics> {
    let decision_map = decisions
        .iter()
        .map(|decision| (decision.id.clone(), decision.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut by_regime = BTreeMap::<Regime, (Vec<DecisionRecord>, Vec<OutcomeRecord>)>::new();
    for outcome in outcomes {
        let entry = by_regime
            .entry(outcome.regime)
            .or_insert_with(|| (Vec::new(), Vec::new()));
        if let Some(decision) = decision_map.get(&outcome.decision_id) {
            entry.0.push(decision.clone());
        }
        entry.1.push(outcome.clone());
    }

    by_regime
        .into_iter()
        .map(
            |(regime, (regime_decisions, regime_outcomes))| RegimeMetrics {
                regime,
                trade_metrics: compute_trade_metrics(&regime_outcomes),
                decision_metrics: compute_decision_metrics(&regime_decisions, &regime_outcomes),
                no_trade_metrics: compute_no_trade_metrics(&regime_outcomes),
                risk_metrics: compute_risk_metrics(&regime_decisions, &regime_outcomes),
            },
        )
        .collect()
}

pub fn compute_persona_metrics(outcomes: &[OutcomeRecord]) -> Vec<PersonaFoldMetrics> {
    let mut by_persona = BTreeMap::<String, PersonaAccumulator>::new();
    for outcome in outcomes {
        let outcome_effect = if outcome.executed {
            outcome.realized_net_return_pct
        } else {
            outcome.avoided_loss_score + outcome.missed_gain_penalty
        };
        for attribution in &outcome.attribution_records {
            let entry = by_persona
                .entry(attribution.persona_id.clone())
                .or_insert_with(PersonaAccumulator::default);
            entry.selected_count += usize::from(attribution.selected_for_decision);
            entry.supported_final_count += usize::from(matches!(
                attribution.counterfactual_role,
                crate::backtest::CounterfactualRole::SupportedFinalDecision
                    | crate::backtest::CounterfactualRole::RiskVetoAligned
            ));
            entry.opposed_final_count += usize::from(matches!(
                attribution.counterfactual_role,
                crate::backtest::CounterfactualRole::OpposedFinalDecision
                    | crate::backtest::CounterfactualRole::RiskVetoOpposed
            ));
            entry.forced_contrarian_count += usize::from(
                attribution.counterfactual_role
                    == crate::backtest::CounterfactualRole::ForcedContrarian,
            );
            entry.contribution_sum += attribution.contribution_score;
            entry.contribution_count += 1;
            entry.net_attributed_return_pct += attribution.contribution_score * outcome_effect;
            if outcome.signal_confidence >= 0.7
                && attribution.selected_for_decision
                && outcome_effect < 0.0
            {
                entry.high_confidence_miss_count += 1;
            }
        }
        for shadow in &outcome.shadow_outcomes {
            let entry = by_persona
                .entry(shadow.persona_id.clone())
                .or_insert_with(PersonaAccumulator::default);
            entry.shadow_count += 1;
        }
    }

    by_persona
        .into_iter()
        .map(|(persona_id, accumulator)| PersonaFoldMetrics {
            persona_id,
            selected_count: accumulator.selected_count,
            shadow_count: accumulator.shadow_count,
            supported_final_count: accumulator.supported_final_count,
            opposed_final_count: accumulator.opposed_final_count,
            forced_contrarian_count: accumulator.forced_contrarian_count,
            avg_contribution_score: safe_ratio(
                accumulator.contribution_sum,
                accumulator.contribution_count as f64,
            ),
            net_attributed_return_pct: accumulator.net_attributed_return_pct,
            high_confidence_miss_count: accumulator.high_confidence_miss_count,
        })
        .collect()
}

pub fn compute_chair_metrics(decisions: &[DecisionRecord]) -> ChairMetrics {
    ChairMetrics {
        approve_candidate_count: decisions
            .iter()
            .filter(|decision| {
                decision.chair_output.decision == ChairDecisionKind::ApproveCandidate
            })
            .count(),
        reduce_size_count: decisions
            .iter()
            .filter(|decision| {
                decision.chair_output.decision == ChairDecisionKind::ReduceSizeCandidate
            })
            .count(),
        no_trade_count: decisions
            .iter()
            .filter(|decision| decision.chair_output.decision == ChairDecisionKind::NoTrade)
            .count(),
        require_confirm_count: decisions
            .iter()
            .filter(|decision| decision.chair_output.decision == ChairDecisionKind::RequireConfirm)
            .count(),
        groupthink_risk_avg: average(
            &decisions
                .iter()
                .map(|decision| decision.chair_output.groupthink_risk)
                .collect::<Vec<_>>(),
        ),
        disagreement_score_avg: average(
            &decisions
                .iter()
                .map(|decision| decision.chair_output.disagreement_score)
                .collect::<Vec<_>>(),
        ),
        cluster_penalty_avg: average(
            &decisions
                .iter()
                .map(|decision| {
                    if decision
                        .chair_output
                        .reason_codes
                        .contains(&ReasonCode::ClusterPenaltyApplied)
                    {
                        1.0
                    } else {
                        0.0
                    }
                })
                .collect::<Vec<_>>(),
        ),
    }
}

#[derive(Default)]
struct PersonaAccumulator {
    selected_count: usize,
    shadow_count: usize,
    supported_final_count: usize,
    opposed_final_count: usize,
    forced_contrarian_count: usize,
    contribution_sum: f64,
    contribution_count: usize,
    net_attributed_return_pct: f64,
    high_confidence_miss_count: usize,
}

fn average(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn safe_ratio(numerator: f64, denominator: f64) -> f64 {
    if denominator.abs() < 1e-12 {
        0.0
    } else {
        numerator / denominator
    }
}
