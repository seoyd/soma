use serde::{Deserialize, Serialize};

use crate::backtest::{
    BacktestSimulator, CandleSeries, CostModel, NoTradeScoreConfig, Timeframe, TripleBarrierConfig,
};
use crate::chair::ChairEngine;
use crate::core::ReasonCode;
use crate::feature::FeatureEngine;
use crate::league::{CycleRiskSkeptic, MomentumTrendFast, ValueQualityFilter};
use crate::model::{EvaluationMode, ExternalPredictionSignalModel};
use crate::regime::RegimeClassifier;
use crate::risk::RiskGovernor;
use crate::signal::BaselineSignalModel;

use super::dataset::{DatasetExportConfig, DatasetFrame, build_dataset_frame};
use super::feature_schema::FeatureSchema;
use super::leakage::LeakageGuard;
use super::metrics::{
    compute_calibration_metrics, compute_chair_metrics, compute_decision_metrics,
    compute_no_trade_metrics, compute_persona_metrics, compute_regime_metrics,
    compute_risk_metrics, compute_trade_metrics,
};
use super::report::{FoldReport, WalkForwardAggregateMetrics, WalkForwardReport};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WalkForwardConfig {
    pub train_window_bars: usize,
    pub validation_window_bars: Option<usize>,
    pub test_window_bars: usize,
    pub step_bars: usize,
    pub embargo_bars: usize,
    pub min_train_bars: usize,
    pub max_folds: Option<usize>,
    pub allow_partial_last_fold: bool,
}

impl Default for WalkForwardConfig {
    fn default() -> Self {
        Self {
            train_window_bars: 40,
            validation_window_bars: None,
            test_window_bars: 20,
            step_bars: 20,
            embargo_bars: 4,
            min_train_bars: 20,
            max_folds: None,
            allow_partial_last_fold: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WalkForwardFold {
    pub fold_id: usize,
    pub train_start_index: usize,
    pub train_end_index: usize,
    pub validation_start_index: Option<usize>,
    pub validation_end_index: Option<usize>,
    pub test_start_index: usize,
    pub test_end_index: usize,
    pub embargo_start_index: Option<usize>,
    pub embargo_end_index: Option<usize>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WalkForwardSplit {
    pub symbol: String,
    pub timeframe: Timeframe,
    pub folds: Vec<WalkForwardFold>,
    pub reason_codes: Vec<ReasonCode>,
}

impl WalkForwardSplit {
    pub fn generate(series: &CandleSeries, config: WalkForwardConfig) -> Self {
        let mut folds = Vec::new();
        let mut reason_codes = Vec::new();

        let validation_window = config.validation_window_bars.unwrap_or(0);
        let minimum_required = config
            .train_window_bars
            .max(config.min_train_bars)
            .saturating_add(validation_window)
            .saturating_add(config.embargo_bars)
            .saturating_add(config.test_window_bars);
        if series.len() < minimum_required || config.step_bars == 0 || config.test_window_bars == 0
        {
            reason_codes.push(ReasonCode::WalkForwardInsufficientData);
            return Self {
                symbol: series.symbol.clone(),
                timeframe: series.timeframe,
                folds,
                reason_codes,
            };
        }

        let mut train_start_index = 0usize;
        while train_start_index < series.len() {
            if config
                .max_folds
                .map(|limit| folds.len() >= limit)
                .unwrap_or(false)
            {
                break;
            }

            let train_end_index = train_start_index + config.train_window_bars.saturating_sub(1);
            if train_end_index >= series.len()
                || train_end_index + 1 - train_start_index < config.min_train_bars
            {
                break;
            }

            let (validation_start_index, validation_end_index, cursor_after_validation) =
                if validation_window > 0 {
                    let start = train_end_index + 1;
                    let end = start + validation_window.saturating_sub(1);
                    if end >= series.len() {
                        break;
                    }
                    (Some(start), Some(end), end + 1)
                } else {
                    (None, None, train_end_index + 1)
                };

            let (embargo_start_index, embargo_end_index, test_start_index) =
                if config.embargo_bars > 0 {
                    let embargo_start = cursor_after_validation;
                    let embargo_end = embargo_start + config.embargo_bars.saturating_sub(1);
                    if embargo_end >= series.len() {
                        break;
                    }
                    (Some(embargo_start), Some(embargo_end), embargo_end + 1)
                } else {
                    (None, None, cursor_after_validation)
                };

            if test_start_index >= series.len() {
                break;
            }

            let remaining_test_bars = series.len() - test_start_index;
            let effective_test_bars = if remaining_test_bars >= config.test_window_bars {
                config.test_window_bars
            } else if config.allow_partial_last_fold {
                remaining_test_bars
            } else {
                break;
            };
            if effective_test_bars == 0 {
                break;
            }
            let test_end_index = test_start_index + effective_test_bars - 1;

            let mut fold_reason_codes = vec![ReasonCode::WalkForwardFoldGenerated];
            if embargo_start_index.is_some() {
                fold_reason_codes.push(ReasonCode::EmbargoApplied);
            }

            folds.push(WalkForwardFold {
                fold_id: folds.len(),
                train_start_index,
                train_end_index,
                validation_start_index,
                validation_end_index,
                test_start_index,
                test_end_index,
                embargo_start_index,
                embargo_end_index,
                reason_codes: fold_reason_codes,
            });

            train_start_index = train_start_index.saturating_add(config.step_bars);
        }

        if folds.is_empty() {
            reason_codes.push(ReasonCode::WalkForwardInsufficientData);
        }

        Self {
            symbol: series.symbol.clone(),
            timeframe: series.timeframe,
            folds,
            reason_codes,
        }
    }
}

#[derive(Clone, Debug)]
pub struct WalkForwardEvaluator {
    pub feature_engine: FeatureEngine,
    pub regime_classifier: RegimeClassifier,
    pub baseline_signal: BaselineSignalModel,
    pub external_signal_model: Option<ExternalPredictionSignalModel>,
    pub full_auto: bool,
    pub chair: ChairEngine,
    pub governor: RiskGovernor,
    pub momentum: MomentumTrendFast,
    pub value: ValueQualityFilter,
    pub skeptic: CycleRiskSkeptic,
    pub triple_barrier_config: TripleBarrierConfig,
    pub cost_model: CostModel,
    pub no_trade_score_config: NoTradeScoreConfig,
}

impl Default for WalkForwardEvaluator {
    fn default() -> Self {
        let simulator = BacktestSimulator::default();
        Self {
            feature_engine: simulator.feature_engine,
            regime_classifier: simulator.regime_classifier,
            baseline_signal: simulator.baseline_signal,
            external_signal_model: None,
            full_auto: simulator.config.full_auto,
            chair: simulator.chair,
            governor: simulator.governor,
            momentum: simulator.momentum,
            value: simulator.value,
            skeptic: simulator.skeptic,
            triple_barrier_config: simulator.config.triple_barrier_config,
            cost_model: simulator.config.cost_model,
            no_trade_score_config: simulator.config.no_trade_score_config,
        }
    }
}

impl WalkForwardEvaluator {
    pub fn split(&self, series: &CandleSeries, config: WalkForwardConfig) -> WalkForwardSplit {
        WalkForwardSplit::generate(series, config)
    }

    pub fn export_dataset(
        &self,
        series: &CandleSeries,
        split: &WalkForwardSplit,
        export_config: &DatasetExportConfig,
    ) -> DatasetFrame {
        build_dataset_frame(
            series,
            split,
            &self.feature_engine,
            &self.regime_classifier,
            self.triple_barrier_config,
            export_config,
        )
    }

    pub fn evaluate(&self, series: &CandleSeries, config: WalkForwardConfig) -> WalkForwardReport {
        self.evaluate_with_mode(series, config, EvaluationMode::BaselineSignal)
    }

    pub fn evaluate_with_mode(
        &self,
        series: &CandleSeries,
        config: WalkForwardConfig,
        evaluation_mode: EvaluationMode,
    ) -> WalkForwardReport {
        let split = self.split(series, config);
        let feature_schema = FeatureSchema::from_engine(&self.feature_engine);
        let mut fold_reports = Vec::new();
        let mut all_decisions = Vec::new();
        let mut all_outcomes = Vec::new();

        for fold in &split.folds {
            let leakage_report =
                LeakageGuard::analyze_fold(fold, self.triple_barrier_config.horizon_bars);
            let evaluation_series = slice_series(series, 0, fold.test_end_index);
            let safe_last_entry_index = fold
                .test_end_index
                .saturating_sub(self.triple_barrier_config.horizon_bars);
            let result = self
                .build_simulator(safe_last_entry_index, evaluation_mode, fold.fold_id)
                .run(&evaluation_series);
            let test_start_timestamp = series.candles[fold.test_start_index].timestamp_ms;
            let decision_records = result
                .decision_records
                .into_iter()
                .filter(|record| record.timestamp_ms >= test_start_timestamp)
                .collect::<Vec<_>>();
            let outcome_records = result
                .outcome_records
                .into_iter()
                .filter(|record| record.timestamp_ms >= test_start_timestamp)
                .collect::<Vec<_>>();

            let fold_report = FoldReport {
                fold_id: fold.fold_id,
                fold: fold.clone(),
                train_rows: fold.train_end_index - fold.train_start_index + 1,
                validation_rows: optional_window_len(
                    fold.validation_start_index,
                    fold.validation_end_index,
                ),
                test_rows: fold.test_end_index - fold.test_start_index + 1,
                leakage_report,
                test_trade_metrics: compute_trade_metrics(&outcome_records),
                test_decision_metrics: compute_decision_metrics(
                    &decision_records,
                    &outcome_records,
                ),
                test_no_trade_metrics: compute_no_trade_metrics(&outcome_records),
                test_risk_metrics: compute_risk_metrics(&decision_records, &outcome_records),
                calibration_metrics: compute_calibration_metrics(
                    &decision_records,
                    &outcome_records,
                ),
                regime_metrics: compute_regime_metrics(&decision_records, &outcome_records),
                persona_metrics: compute_persona_metrics(&outcome_records),
                chair_metrics: compute_chair_metrics(&decision_records),
                reason_codes: {
                    let mut codes = fold.reason_codes.clone();
                    codes.push(ReasonCode::WalkForwardEvaluated);
                    codes
                },
            };

            all_decisions.extend(decision_records);
            all_outcomes.extend(outcome_records);
            fold_reports.push(fold_report);
        }

        let aggregate_metrics = WalkForwardAggregateMetrics {
            trade_metrics: compute_trade_metrics(&all_outcomes),
            decision_metrics: compute_decision_metrics(&all_decisions, &all_outcomes),
            no_trade_metrics: compute_no_trade_metrics(&all_outcomes),
            risk_metrics: compute_risk_metrics(&all_decisions, &all_outcomes),
            calibration_metrics: compute_calibration_metrics(&all_decisions, &all_outcomes),
            regime_metrics: compute_regime_metrics(&all_decisions, &all_outcomes),
            persona_metrics: compute_persona_metrics(&all_outcomes),
            chair_metrics: compute_chair_metrics(&all_decisions),
        };

        let mut reason_codes = split.reason_codes.clone();
        match feature_schema.validate_feature_names(&self.feature_engine.feature_names()) {
            Ok(()) => reason_codes.push(ReasonCode::FeatureSchemaValidated),
            Err(mut codes) => reason_codes.append(&mut codes),
        }
        reason_codes.push(ReasonCode::WalkForwardEvaluated);

        WalkForwardReport {
            symbol: series.symbol.clone(),
            timeframe: series.timeframe,
            config,
            folds: fold_reports,
            aggregate_metrics,
            feature_schema,
            reason_codes,
        }
    }

    fn build_simulator(
        &self,
        safe_test_steps: usize,
        evaluation_mode: EvaluationMode,
        fold_id: usize,
    ) -> BacktestSimulator {
        let mut simulator = BacktestSimulator::default();
        simulator.feature_engine = self.feature_engine.clone();
        simulator.regime_classifier = self.regime_classifier.clone();
        simulator.baseline_signal = self.baseline_signal.clone();
        simulator.evaluation_mode = evaluation_mode;
        simulator.external_signal_model = self
            .external_signal_model
            .as_ref()
            .map(|model| model.for_fold(fold_id));
        simulator.config.full_auto = self.full_auto;
        simulator.chair = self.chair.clone();
        simulator.governor = self.governor.clone();
        simulator.momentum = self.momentum.clone();
        simulator.value = self.value.clone();
        simulator.skeptic = self.skeptic.clone();
        simulator.config.triple_barrier_config = self.triple_barrier_config;
        simulator.config.cost_model = self.cost_model;
        simulator.config.no_trade_score_config = self.no_trade_score_config;
        simulator.config.max_steps = Some(safe_test_steps);
        simulator
    }
}

fn optional_window_len(start_index: Option<usize>, end_index: Option<usize>) -> usize {
    match (start_index, end_index) {
        (Some(start), Some(end)) if start <= end => end - start + 1,
        _ => 0,
    }
}

fn slice_series(series: &CandleSeries, start_index: usize, end_index: usize) -> CandleSeries {
    CandleSeries {
        symbol: series.symbol.clone(),
        timeframe: series.timeframe,
        candles: series.candles[start_index..=end_index].to_vec(),
    }
}
