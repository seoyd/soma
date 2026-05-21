use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::chair::ChairEngine;
use crate::core::{
    AuditEvent, AuditEventType, FeatureVector, MarketSnapshot, ReasonCode, RiskDecision,
    RiskDecisionKind, RiskSnapshot, Side, SignalOutput, Stance, TradeProposal, build_audit_event,
    stable_hash,
};
use crate::feature::FeatureEngine;
use crate::league::{
    CycleRiskSkeptic, MomentumTrendFast, Persona, ValueQualityFilter, horizon_from_bars,
};
use crate::model::{EvaluationMode, ExternalPredictionSignalModel, conservative_no_trade_signal};
use crate::paper::{Broker, PaperBroker};
use crate::regime::RegimeClassifier;
use crate::risk::RiskGovernor;
use crate::signal::{BaselineSignalModel, MockSignalEngine, derive_features};

use super::{
    AttributionRecord, CandleSeries, CostModel, CounterfactualRole, DecisionRecord,
    NoTradeEvaluation, OutcomeRecord, ShadowOutcomeRecord, TripleBarrierConfig,
    TripleBarrierOutcome, TripleBarrierResult, evaluate_triple_barrier,
};

#[derive(Clone, Debug, PartialEq)]
pub struct SimulationResult {
    pub features: FeatureVector,
    pub signal: SignalOutput,
    pub votes: Vec<crate::core::InvestorVote>,
    pub chair_output: crate::core::ChairOutput,
    pub trade_proposal: Option<TradeProposal>,
    pub risk_decision: RiskDecision,
    pub paper_order: Option<crate::core::PaperOrder>,
    pub audit_events: Vec<AuditEvent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NoTradeScoreConfig {
    pub avoided_loss_weight: f64,
    pub missed_gain_weight: f64,
}

impl Default for NoTradeScoreConfig {
    fn default() -> Self {
        Self {
            avoided_loss_weight: 0.7,
            missed_gain_weight: 0.2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BacktestConfig {
    pub triple_barrier_config: TripleBarrierConfig,
    pub cost_model: CostModel,
    pub no_trade_score_config: NoTradeScoreConfig,
    pub full_auto: bool,
    pub max_steps: Option<usize>,
    pub starting_equity: Option<f64>,
    pub paper_mode: bool,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            triple_barrier_config: TripleBarrierConfig {
                take_profit_pct: 0.02,
                stop_loss_pct: 0.01,
                horizon_bars: 8,
                fee_bps: 2.0,
                slippage_bps: 2.0,
                side: Side::Long,
                use_high_low_intrabar: true,
            },
            cost_model: CostModel {
                fee_bps: 2.0,
                slippage_bps: 2.0,
                spread_bps: Some(2.0),
                min_cost_bps: None,
            },
            no_trade_score_config: NoTradeScoreConfig::default(),
            full_auto: false,
            max_steps: None,
            starting_equity: Some(1.0),
            paper_mode: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BacktestResult {
    pub total_decisions: usize,
    pub executed_trades: usize,
    pub denied_trades: usize,
    pub no_trades: usize,
    pub wins: usize,
    pub losses: usize,
    pub neutrals: usize,
    pub gross_return_pct: f64,
    pub net_return_pct: f64,
    pub max_drawdown_pct: f64,
    pub profit_factor: Option<f64>,
    pub reason_codes: Vec<ReasonCode>,
    pub decision_records: Vec<DecisionRecord>,
    pub outcome_records: Vec<OutcomeRecord>,
}

#[derive(Clone, Debug)]
pub struct BacktestSimulator {
    pub config: BacktestConfig,
    pub feature_engine: FeatureEngine,
    pub regime_classifier: RegimeClassifier,
    pub baseline_signal: BaselineSignalModel,
    pub evaluation_mode: EvaluationMode,
    pub external_signal_model: Option<ExternalPredictionSignalModel>,
    pub chair: ChairEngine,
    pub governor: RiskGovernor,
    pub momentum: MomentumTrendFast,
    pub value: ValueQualityFilter,
    pub skeptic: CycleRiskSkeptic,
}

impl Default for BacktestSimulator {
    fn default() -> Self {
        Self {
            config: BacktestConfig::default(),
            feature_engine: FeatureEngine::default(),
            regime_classifier: RegimeClassifier::default(),
            baseline_signal: BaselineSignalModel::default(),
            evaluation_mode: EvaluationMode::BaselineSignal,
            external_signal_model: None,
            chair: ChairEngine::default(),
            governor: RiskGovernor::default(),
            momentum: MomentumTrendFast::default(),
            value: ValueQualityFilter::default(),
            skeptic: CycleRiskSkeptic::default(),
        }
    }
}

impl BacktestSimulator {
    pub fn run(&self, series: &CandleSeries) -> BacktestResult {
        let mut broker = PaperBroker::default();
        let mut decision_records = Vec::new();
        let mut outcome_records = Vec::new();
        let mut total_decisions = 0usize;
        let mut executed_trades = 0usize;
        let mut denied_trades = 0usize;
        let mut no_trades = 0usize;
        let mut wins = 0usize;
        let mut losses = 0usize;
        let mut neutrals = 0usize;
        let mut gross_return_pct = 0.0;
        let mut net_return_pct = 0.0;
        let mut consecutive_losses = 0u32;
        let mut equity = self.config.starting_equity.unwrap_or(1.0).max(1e-9);
        let mut peak_equity = equity;
        let mut max_drawdown_pct: f64 = 0.0;
        let max_steps = self
            .config
            .max_steps
            .unwrap_or_else(|| series.len().saturating_sub(1));

        for current_index in 1..series.len() {
            if total_decisions >= max_steps {
                break;
            }
            let cursor = match series.replay_cursor(current_index) {
                Some(cursor) => cursor,
                None => break,
            };
            let raw_market = cursor.market_snapshot();
            let feature_vector = self.feature_engine.build_at(series, current_index);
            let regime_decision = self.regime_classifier.classify(
                &feature_vector,
                cursor.lookback_window(self.feature_engine.config.min_required_bars.max(20)),
            );
            let mut market = raw_market.clone();
            market.regime = regime_decision.regime;
            market.data_quality_score = feature_vector.data_quality_score;
            let risk_snapshot = RiskSnapshot {
                daily_pnl_pct: net_return_pct,
                consecutive_losses,
                current_positions_count: 0,
                total_exposure_pct: 0.0,
                symbol_exposure_pct: 0.0,
                api_health_score: 1.0,
                data_quality_score: feature_vector.data_quality_score,
            };
            let signal = match self.evaluation_mode {
                EvaluationMode::BaselineSignal => self.baseline_signal.evaluate(
                    &feature_vector,
                    &regime_decision,
                    &self.config.cost_model,
                ),
                EvaluationMode::ExternalPrediction => self
                    .external_signal_model
                    .as_ref()
                    .map(|model| {
                        model.signal_for(
                            &feature_vector.symbol,
                            feature_vector.timestamp_ms,
                            feature_vector.timeframe,
                            None,
                            None,
                        )
                    })
                    .unwrap_or_else(|| {
                        conservative_no_trade_signal(
                            &feature_vector.symbol,
                            "external_prediction_missing_model",
                            self.config.triple_barrier_config.horizon_bars as u32,
                        )
                    }),
            };
            let votes = self.league_votes(&market, &signal);
            let chair_input = crate::core::ChairInput {
                market: market.clone(),
                signal: signal.clone(),
                votes: votes.clone(),
                full_auto: self.config.full_auto,
            };
            let chair_output = self.chair.evaluate(&chair_input);
            let trade_proposal = self
                .chair
                .build_trade_proposal(&market, &signal, &chair_output)
                .map(|proposal| self.normalize_trade_proposal(proposal, &market, &signal));
            let risk_decision = self.governor.evaluate(
                &market,
                &risk_snapshot,
                trade_proposal.as_ref(),
                market.timestamp_ms,
            );
            let paper_order = risk_decision.approved_order_plan.clone().map(|plan| {
                broker.submit_paper_order(
                    plan,
                    market.timestamp_ms,
                    vec![ReasonCode::PaperExecutionOnly, ReasonCode::ApprovePaperOnly],
                )
            });

            let decision_record = build_decision_record(
                &market,
                &signal,
                votes.clone(),
                chair_output.clone(),
                risk_decision.clone(),
                trade_proposal.clone(),
                paper_order.as_ref().map(|order| order.order_id.clone()),
            );
            let outcome_record =
                self.evaluate_outcome(series, current_index, &market, &signal, &decision_record);

            if outcome_record.executed {
                executed_trades += 1;
                if let Some(result) = outcome_record.triple_barrier_result.as_ref() {
                    gross_return_pct += result.gross_return_pct;
                    net_return_pct += result.net_return_pct;
                    equity *= 1.0 + result.net_return_pct;
                    peak_equity = peak_equity.max(equity);
                    max_drawdown_pct =
                        max_drawdown_pct.max((1.0 - equity / peak_equity.max(1e-9)).max(0.0));
                    match result.outcome {
                        TripleBarrierOutcome::Win => {
                            wins += 1;
                            consecutive_losses = 0;
                        }
                        TripleBarrierOutcome::Loss => {
                            losses += 1;
                            consecutive_losses += 1;
                        }
                        TripleBarrierOutcome::Neutral => {
                            neutrals += 1;
                            consecutive_losses = 0;
                        }
                        TripleBarrierOutcome::NoData => {}
                    }
                }
            } else if outcome_record.denied_by_risk {
                denied_trades += 1;
            } else if outcome_record.no_trade {
                no_trades += 1;
            }

            total_decisions += 1;
            decision_records.push(decision_record);
            outcome_records.push(outcome_record);
        }

        let total_gains: f64 = outcome_records
            .iter()
            .filter_map(|record| record.triple_barrier_result.as_ref())
            .map(|result| result.net_return_pct)
            .filter(|net| *net > 0.0)
            .sum();
        let total_losses_abs: f64 = outcome_records
            .iter()
            .filter_map(|record| record.triple_barrier_result.as_ref())
            .map(|result| result.net_return_pct)
            .filter(|net| *net < 0.0)
            .map(f64::abs)
            .sum();

        BacktestResult {
            total_decisions,
            executed_trades,
            denied_trades,
            no_trades,
            wins,
            losses,
            neutrals,
            gross_return_pct,
            net_return_pct,
            max_drawdown_pct,
            profit_factor: if total_losses_abs > 0.0 {
                Some(total_gains / total_losses_abs)
            } else {
                None
            },
            reason_codes: vec![
                ReasonCode::DeterministicPath,
                ReasonCode::PaperExecutionOnly,
                ReasonCode::BacktestReplay,
            ],
            decision_records,
            outcome_records,
        }
    }

    fn league_votes(
        &self,
        market: &MarketSnapshot,
        signal: &SignalOutput,
    ) -> Vec<crate::core::InvestorVote> {
        vec![
            self.momentum.vote(market, signal),
            self.value.vote(market, signal),
            self.skeptic.vote(market, signal),
        ]
    }

    fn normalize_trade_proposal(
        &self,
        mut proposal: TradeProposal,
        market: &MarketSnapshot,
        signal: &SignalOutput,
    ) -> TradeProposal {
        proposal.max_slippage_bps = self
            .config
            .cost_model
            .slippage_bps
            .max(market.spread_bps.max(1.0));
        proposal.expected_edge_after_cost = self
            .config
            .cost_model
            .expected_edge_after_cost(signal.expected_return.max(0.0));
        proposal
    }

    fn evaluate_outcome(
        &self,
        series: &CandleSeries,
        entry_index: usize,
        market: &MarketSnapshot,
        signal: &SignalOutput,
        decision: &DecisionRecord,
    ) -> OutcomeRecord {
        let executed = decision.selected_for_execution;
        let denied_by_risk = decision.risk_decision.kind == RiskDecisionKind::Deny
            && decision.trade_proposal.is_some();
        let no_trade = !executed && !denied_by_risk;

        let triple_barrier_result = if executed {
            decision.trade_proposal.as_ref().map(|proposal| {
                evaluate_triple_barrier(
                    series,
                    entry_index,
                    proposal.entry_price_hint,
                    self.effective_barrier_config(proposal, signal),
                )
            })
        } else {
            None
        };

        let hypothetical_result = if denied_by_risk || no_trade {
            self.counterfactual_proposal(market, signal, decision)
                .map(|proposal| {
                    evaluate_triple_barrier(
                        series,
                        entry_index,
                        proposal.entry_price_hint,
                        self.effective_barrier_config(&proposal, signal),
                    )
                })
        } else {
            None
        };

        let no_trade_evaluation = if denied_by_risk || no_trade {
            evaluate_no_trade_counterfactual(
                hypothetical_result.as_ref(),
                denied_by_risk,
                self.config.no_trade_score_config,
            )
        } else {
            NoTradeEvaluation {
                hypothetical_result: None,
                avoided_loss_score: 0.0,
                missed_gain_penalty: 0.0,
                reason_codes: Vec::new(),
            }
        };

        let attribution_records = build_attribution_records(
            &decision.investor_votes,
            &decision.chair_output,
            decision.trade_proposal.as_ref(),
            denied_by_risk,
            no_trade,
            triple_barrier_result.as_ref(),
            hypothetical_result.as_ref(),
            no_trade_evaluation.avoided_loss_score,
            no_trade_evaluation.missed_gain_penalty,
        );
        let shadow_outcomes = build_shadow_outcomes(
            &decision.investor_votes,
            &decision.chair_output.selected_speakers,
            hypothetical_result.as_ref(),
            decision.trade_proposal.as_ref(),
        );

        let mut reason_codes = decision.reason_codes.clone();
        if denied_by_risk {
            reason_codes.push(ReasonCode::RiskDeniedCounterfactual);
        }
        if no_trade {
            reason_codes.push(ReasonCode::NoTradeCounterfactual);
        }
        reason_codes.extend(no_trade_evaluation.reason_codes.iter().cloned());

        let realized_net_return_pct = triple_barrier_result
            .as_ref()
            .map(|result| result.net_return_pct)
            .unwrap_or(0.0);

        OutcomeRecord {
            decision_id: decision.id.clone(),
            symbol: decision.symbol.clone(),
            timestamp_ms: decision.timestamp_ms,
            regime: market.regime,
            horizon: horizon_from_bars(signal.horizon_bars),
            signal_confidence: signal.confidence,
            executed,
            denied_by_risk,
            no_trade,
            triple_barrier_result,
            hypothetical_result: no_trade_evaluation
                .hypothetical_result
                .or(hypothetical_result),
            realized_net_return_pct,
            avoided_loss_score: no_trade_evaluation.avoided_loss_score,
            missed_gain_penalty: no_trade_evaluation.missed_gain_penalty,
            attribution_records,
            shadow_outcomes,
            reason_codes,
        }
    }

    fn counterfactual_proposal(
        &self,
        market: &MarketSnapshot,
        signal: &SignalOutput,
        decision: &DecisionRecord,
    ) -> Option<TradeProposal> {
        if let Some(proposal) = decision.trade_proposal.clone() {
            return Some(self.normalize_trade_proposal(proposal, market, signal));
        }

        let strongest_vote = decision
            .investor_votes
            .iter()
            .filter(|vote| matches!(vote.stance, Stance::Buy | Stance::Sell))
            .max_by(|left, right| {
                (left.voice_power * left.conviction)
                    .total_cmp(&(right.voice_power * right.conviction))
            })?;

        let stop_distance = signal.expected_drawdown.clamp(0.002, 0.05);
        let take_profit_distance = signal
            .expected_return
            .abs()
            .max(stop_distance * 1.5)
            .min(0.10);
        let side = match strongest_vote.stance {
            Stance::Buy => Side::Long,
            Stance::Sell => Side::Short,
            Stance::NoTrade | Stance::Abstain => return None,
        };

        let (stop_loss, take_profit) = match side {
            Side::Long => (
                Some(market.price * (1.0 - stop_distance)),
                Some(market.price * (1.0 + take_profit_distance)),
            ),
            Side::Short => (
                Some(market.price * (1.0 + stop_distance)),
                Some(market.price * (1.0 - take_profit_distance)),
            ),
        };

        Some(TradeProposal {
            symbol: market.symbol.clone(),
            side,
            quantity_hint: (strongest_vote.voice_power * strongest_vote.conviction)
                .clamp(0.05, 1.0),
            entry_price_hint: market.price,
            stop_loss,
            take_profit,
            max_slippage_bps: self
                .config
                .cost_model
                .slippage_bps
                .max(market.spread_bps.max(1.0)),
            expected_edge_after_cost: self
                .config
                .cost_model
                .expected_edge_after_cost(signal.expected_return.abs()),
            confidence: signal.confidence,
            source_chair_output: decision.chair_output.clone(),
        })
    }

    fn effective_barrier_config(
        &self,
        proposal: &TradeProposal,
        signal: &SignalOutput,
    ) -> TripleBarrierConfig {
        let take_profit_pct = proposal
            .take_profit
            .map(|take_profit| {
                ((take_profit - proposal.entry_price_hint).abs())
                    / proposal.entry_price_hint.max(1e-9)
            })
            .unwrap_or(self.config.triple_barrier_config.take_profit_pct);
        let stop_loss_pct = proposal
            .stop_loss
            .map(|stop_loss| {
                ((stop_loss - proposal.entry_price_hint).abs())
                    / proposal.entry_price_hint.max(1e-9)
            })
            .unwrap_or(self.config.triple_barrier_config.stop_loss_pct);

        TripleBarrierConfig {
            take_profit_pct: if take_profit_pct > 0.0 {
                take_profit_pct
            } else {
                self.config
                    .triple_barrier_config
                    .take_profit_pct
                    .max(signal.expected_return.abs())
            },
            stop_loss_pct: if stop_loss_pct > 0.0 {
                stop_loss_pct
            } else {
                self.config
                    .triple_barrier_config
                    .stop_loss_pct
                    .max(signal.expected_drawdown.abs())
            },
            horizon_bars: self
                .config
                .triple_barrier_config
                .horizon_bars
                .max(signal.horizon_bars as usize),
            fee_bps: if self.config.triple_barrier_config.fee_bps > 0.0 {
                self.config.triple_barrier_config.fee_bps
            } else {
                self.config.cost_model.fee_bps
            },
            slippage_bps: if self.config.triple_barrier_config.slippage_bps > 0.0 {
                self.config.triple_barrier_config.slippage_bps
            } else {
                self.config.cost_model.slippage_bps
            },
            side: proposal.side,
            use_high_low_intrabar: self.config.triple_barrier_config.use_high_low_intrabar,
        }
    }
}

pub fn simulate_paper_cycle(
    market: &MarketSnapshot,
    risk_snapshot: &RiskSnapshot,
    signal_engine: &MockSignalEngine,
    chair: &ChairEngine,
    governor: &RiskGovernor,
    broker: &mut PaperBroker,
    full_auto: bool,
) -> SimulationResult {
    let features = derive_features(market);
    let signal = signal_engine.evaluate_with_features(market, &features);
    let votes = vec![
        MomentumTrendFast::default().vote(market, &signal),
        ValueQualityFilter::default().vote(market, &signal),
        CycleRiskSkeptic::default().vote(market, &signal),
    ];
    let chair_input = crate::core::ChairInput {
        market: market.clone(),
        signal: signal.clone(),
        votes: votes.clone(),
        full_auto,
    };
    let chair_output = chair.evaluate(&chair_input);
    let trade_proposal = chair.build_trade_proposal(market, &signal, &chair_output);
    let risk_decision = governor.evaluate(
        market,
        risk_snapshot,
        trade_proposal.as_ref(),
        market.timestamp_ms,
    );

    let mut audit_events = vec![
        build_audit_event(
            market.timestamp_ms,
            AuditEventType::SignalEvaluated,
            &format!("{:?}{:?}", market, features),
            "signal generated",
            vec![ReasonCode::DeterministicPath],
            BTreeMap::from([
                ("expected_return".to_string(), signal.expected_return),
                ("confidence".to_string(), signal.confidence),
                (
                    "no_trade_probability".to_string(),
                    signal.no_trade_probability,
                ),
            ]),
        ),
        build_audit_event(
            market.timestamp_ms,
            AuditEventType::ChairEvaluated,
            &format!("{:?}{:?}", votes, chair_output),
            "chair evaluated votes",
            chair_output.reason_codes.clone(),
            BTreeMap::from([
                ("council_score".to_string(), chair_output.council_score),
                ("groupthink_risk".to_string(), chair_output.groupthink_risk),
                (
                    "disagreement_score".to_string(),
                    chair_output.disagreement_score,
                ),
            ]),
        ),
        build_audit_event(
            market.timestamp_ms,
            AuditEventType::RiskEvaluated,
            &format!("{:?}{:?}", trade_proposal, risk_decision),
            "risk governor evaluated candidate",
            risk_decision.reason_codes.clone(),
            BTreeMap::from([
                ("daily_pnl_pct".to_string(), risk_snapshot.daily_pnl_pct),
                (
                    "total_exposure_pct".to_string(),
                    risk_snapshot.total_exposure_pct,
                ),
                (
                    "proposal_edge".to_string(),
                    trade_proposal
                        .as_ref()
                        .map(|proposal| proposal.expected_edge_after_cost)
                        .unwrap_or(0.0),
                ),
            ]),
        ),
    ];

    let paper_order = risk_decision.approved_order_plan.clone().map(|plan| {
        let order = broker.submit_paper_order(
            plan,
            market.timestamp_ms,
            vec![ReasonCode::PaperExecutionOnly, ReasonCode::ApprovePaperOnly],
        );
        let order_audit = build_audit_event(
            market.timestamp_ms,
            AuditEventType::PaperOrderCreated,
            &format!("{:?}", order),
            "paper order created",
            order.reason_codes.clone(),
            BTreeMap::from([
                ("quantity".to_string(), order.quantity),
                ("entry_price".to_string(), order.entry_price),
            ]),
        );
        broker.ledger.record_audit(order_audit.clone());
        audit_events.push(order_audit);
        order
    });

    let completion_audit = build_audit_event(
        market.timestamp_ms,
        AuditEventType::SimulationCompleted,
        &format!("{:?}{:?}", risk_decision, paper_order),
        "simulation completed",
        vec![ReasonCode::DeterministicPath],
        BTreeMap::from([("audit_events".to_string(), audit_events.len() as f64)]),
    );
    broker.ledger.record_audit(completion_audit.clone());
    audit_events.push(completion_audit);

    SimulationResult {
        features,
        signal,
        votes,
        chair_output,
        trade_proposal,
        risk_decision,
        paper_order,
        audit_events,
    }
}

fn build_decision_record(
    market: &MarketSnapshot,
    signal: &SignalOutput,
    votes: Vec<crate::core::InvestorVote>,
    chair_output: crate::core::ChairOutput,
    risk_decision: RiskDecision,
    trade_proposal: Option<TradeProposal>,
    paper_order_id: Option<String>,
) -> DecisionRecord {
    let id_material = format!(
        "{}:{}:{:?}:{:?}",
        market.symbol, market.timestamp_ms, chair_output.decision, risk_decision.kind
    );
    let mut reason_codes = chair_output.reason_codes.clone();
    for reason in &risk_decision.reason_codes {
        if !reason_codes.contains(reason) {
            reason_codes.push(reason.clone());
        }
    }

    DecisionRecord {
        id: format!("decision-{:#016x}", stable_hash(&id_material)),
        timestamp_ms: market.timestamp_ms,
        symbol: market.symbol.clone(),
        signal_output: signal.clone(),
        investor_votes: votes,
        chair_output,
        risk_decision: risk_decision.clone(),
        trade_proposal: trade_proposal.clone(),
        selected_for_execution: risk_decision.kind == RiskDecisionKind::ApprovePaper,
        paper_order_id,
        reason_codes,
        audit_event_id: risk_decision.audit_id.clone(),
    }
}

pub fn evaluate_no_trade_counterfactual(
    hypothetical_result: Option<&TripleBarrierResult>,
    denied_by_risk: bool,
    config: NoTradeScoreConfig,
) -> NoTradeEvaluation {
    let mut reason_codes = vec![ReasonCode::CounterfactualEvaluated];
    if denied_by_risk {
        reason_codes.push(ReasonCode::RiskDeniedCounterfactual);
    } else {
        reason_codes.push(ReasonCode::NoTradeCounterfactual);
    }
    let Some(result) = hypothetical_result.cloned() else {
        return NoTradeEvaluation {
            hypothetical_result: None,
            avoided_loss_score: 0.0,
            missed_gain_penalty: 0.0,
            reason_codes,
        };
    };

    if result.net_return_pct < 0.0 || result.outcome == TripleBarrierOutcome::Loss {
        reason_codes.push(ReasonCode::PositiveSilenceValue);
        reason_codes.push(ReasonCode::AvoidedLossRecorded);
        if denied_by_risk {
            reason_codes.push(ReasonCode::DefensiveAttribution);
        }
        NoTradeEvaluation {
            hypothetical_result: Some(result.clone()),
            avoided_loss_score: config.avoided_loss_weight.max(0.0) * result.net_return_pct.abs(),
            missed_gain_penalty: 0.0,
            reason_codes,
        }
    } else if result.net_return_pct > 0.0 || result.outcome == TripleBarrierOutcome::Win {
        reason_codes.push(ReasonCode::NegativeSilenceValue);
        reason_codes.push(ReasonCode::MissedGainRecorded);
        if denied_by_risk {
            reason_codes.push(ReasonCode::OpportunityCostRecorded);
        }
        NoTradeEvaluation {
            hypothetical_result: Some(result.clone()),
            avoided_loss_score: 0.0,
            missed_gain_penalty: -config.missed_gain_weight.max(0.0)
                * result.net_return_pct.max(0.0),
            reason_codes,
        }
    } else {
        NoTradeEvaluation {
            hypothetical_result: Some(result),
            avoided_loss_score: 0.0,
            missed_gain_penalty: 0.0,
            reason_codes,
        }
    }
}

fn build_attribution_records(
    votes: &[crate::core::InvestorVote],
    chair_output: &crate::core::ChairOutput,
    trade_proposal: Option<&TradeProposal>,
    denied_by_risk: bool,
    no_trade: bool,
    executed_result: Option<&TripleBarrierResult>,
    hypothetical_result: Option<&TripleBarrierResult>,
    avoided_loss_score: f64,
    missed_gain_penalty: f64,
) -> Vec<AttributionRecord> {
    let final_side = trade_proposal.map(|proposal| proposal.side);
    votes
        .iter()
        .map(|vote| {
            let selected_for_decision = chair_output.selected_speakers.contains(&vote.persona_id);
            let forced_contrarian = chair_output.forced_contrarian && selected_for_decision;
            let counterfactual_role = if denied_by_risk {
                if vote.stance == Stance::NoTrade || vote.veto {
                    CounterfactualRole::RiskVetoAligned
                } else {
                    CounterfactualRole::RiskVetoOpposed
                }
            } else if forced_contrarian {
                CounterfactualRole::ForcedContrarian
            } else if no_trade {
                if matches!(vote.stance, Stance::NoTrade | Stance::Abstain) {
                    CounterfactualRole::SupportedFinalDecision
                } else {
                    CounterfactualRole::OpposedFinalDecision
                }
            } else if aligns_with_trade(vote.stance, final_side) {
                CounterfactualRole::SupportedFinalDecision
            } else {
                CounterfactualRole::OpposedFinalDecision
            };

            let raw_strength = vote.voice_power * vote.conviction;
            let contribution_score = match counterfactual_role {
                CounterfactualRole::SupportedFinalDecision => raw_strength,
                CounterfactualRole::ForcedContrarian => raw_strength * 0.5,
                CounterfactualRole::OpposedFinalDecision => -raw_strength,
                CounterfactualRole::ShadowOnly => 0.0,
                CounterfactualRole::RiskVetoAligned => {
                    if hypothetical_result
                        .map(|result| {
                            result.net_return_pct < 0.0
                                || result.outcome == TripleBarrierOutcome::Loss
                        })
                        .unwrap_or(false)
                    {
                        raw_strength + avoided_loss_score
                    } else {
                        raw_strength + missed_gain_penalty
                    }
                }
                CounterfactualRole::RiskVetoOpposed => {
                    if hypothetical_result
                        .map(|result| {
                            result.net_return_pct < 0.0
                                || result.outcome == TripleBarrierOutcome::Loss
                        })
                        .unwrap_or(false)
                    {
                        -(raw_strength + avoided_loss_score)
                    } else {
                        raw_strength * 0.25
                    }
                }
            };

            let mut reason_codes = vote.reason_codes.clone();
            if denied_by_risk {
                reason_codes.push(ReasonCode::RiskDeniedCounterfactual);
            } else if no_trade {
                reason_codes.push(ReasonCode::NoTradeCounterfactual);
            } else if executed_result.is_some() {
                reason_codes.push(ReasonCode::PaperFillSimulated);
            }

            AttributionRecord {
                persona_id: vote.persona_id.clone(),
                selected_for_decision,
                stance: vote.stance,
                conviction: vote.conviction,
                voice_power: vote.voice_power,
                contribution_score,
                counterfactual_role,
                reason_codes,
            }
        })
        .collect()
}

fn build_shadow_outcomes(
    votes: &[crate::core::InvestorVote],
    selected_speakers: &[String],
    hypothetical_result: Option<&TripleBarrierResult>,
    trade_proposal: Option<&TradeProposal>,
) -> Vec<ShadowOutcomeRecord> {
    votes
        .iter()
        .filter(|vote| !selected_speakers.contains(&vote.persona_id))
        .map(|vote| ShadowOutcomeRecord {
            persona_id: vote.persona_id.clone(),
            hypothetical_stance: vote.stance,
            hypothetical_result: if matches!(vote.stance, Stance::Buy | Stance::Sell) {
                hypothetical_result.cloned()
            } else {
                None
            },
            would_have_supported_trade: aligns_with_trade(
                vote.stance,
                trade_proposal.map(|proposal| proposal.side),
            ),
            would_have_blocked_trade: vote.veto || vote.stance == Stance::NoTrade,
            evaluation_pending: false,
        })
        .collect()
}

fn aligns_with_trade(stance: Stance, final_side: Option<Side>) -> bool {
    match (stance, final_side) {
        (Stance::Buy, Some(Side::Long)) => true,
        (Stance::Sell, Some(Side::Short)) => true,
        _ => false,
    }
}
