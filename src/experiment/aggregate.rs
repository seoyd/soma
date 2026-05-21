use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, Regime};
use crate::data::DataQualitySeverity;
use crate::eval::{PersonaFoldMetrics, RegimeMetrics};

use super::report_bundle::ExperimentReportBundle;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ExperimentRunKey {
    pub dataset_id: String,
    pub variant_id: String,
    pub experiment_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentRunStatus {
    Passed,
    Failed,
    Skipped,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExperimentRunSummary {
    pub run_key: ExperimentRunKey,
    pub status: ExperimentRunStatus,
    pub manifest_summary: String,
    pub data_quality_score: f64,
    pub data_quality_severity: DataQualitySeverity,
    pub total_decisions: usize,
    pub executed_trades: usize,
    pub denied_trades: usize,
    pub no_trades: usize,
    pub net_return_pct: f64,
    pub max_drawdown_pct: f64,
    pub profit_factor: Option<f64>,
    pub calibration_brier: Option<f64>,
    pub risk_defensive_value: Option<f64>,
    pub external_better: Option<bool>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AggregateBenchmark {
    pub total_runs: usize,
    pub passed_runs: usize,
    pub failed_runs: usize,
    pub skipped_runs: usize,
    pub baseline_runs: usize,
    pub external_runs: usize,
    pub avg_net_return_pct: f64,
    pub median_net_return_pct: f64,
    pub worst_net_return_pct: f64,
    pub avg_max_drawdown_pct: f64,
    pub worst_max_drawdown_pct: f64,
    pub avg_profit_factor: Option<f64>,
    pub avg_no_trade_rate: f64,
    pub avg_denied_rate: f64,
    pub avg_data_quality_score: f64,
    pub reason_codes: Vec<ReasonCode>,
}

impl AggregateBenchmark {
    pub fn to_markdown_table_string(&self) -> String {
        [
            "| metric | value |".to_string(),
            "| --- | ---: |".to_string(),
            format!("| total_runs | {} |", self.total_runs),
            format!("| passed_runs | {} |", self.passed_runs),
            format!("| failed_runs | {} |", self.failed_runs),
            format!("| skipped_runs | {} |", self.skipped_runs),
            format!("| baseline_runs | {} |", self.baseline_runs),
            format!("| external_runs | {} |", self.external_runs),
            format!("| avg_net_return_pct | {:.8} |", self.avg_net_return_pct),
            format!(
                "| median_net_return_pct | {:.8} |",
                self.median_net_return_pct
            ),
            format!(
                "| worst_net_return_pct | {:.8} |",
                self.worst_net_return_pct
            ),
            format!(
                "| avg_max_drawdown_pct | {:.8} |",
                self.avg_max_drawdown_pct
            ),
            format!(
                "| worst_max_drawdown_pct | {:.8} |",
                self.worst_max_drawdown_pct
            ),
            format!(
                "| avg_profit_factor | {} |",
                self.avg_profit_factor
                    .map(|value| format!("{value:.8}"))
                    .unwrap_or_default()
            ),
            format!("| avg_no_trade_rate | {:.8} |", self.avg_no_trade_rate),
            format!("| avg_denied_rate | {:.8} |", self.avg_denied_rate),
            format!(
                "| avg_data_quality_score | {:.8} |",
                self.avg_data_quality_score
            ),
        ]
        .join("\n")
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataQualityAggregate {
    pub dataset_count: usize,
    pub good_count: usize,
    pub warning_count: usize,
    pub bad_count: usize,
    pub unusable_count: usize,
    pub avg_data_quality_score: f64,
    pub worst_dataset_id: Option<String>,
    pub common_reason_codes: Vec<(String, usize)>,
    pub gap_heavy_datasets: Vec<String>,
    pub duplicate_heavy_datasets: Vec<String>,
    pub invalid_ohlc_datasets: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegimeAggregate {
    pub counts_by_regime: BTreeMap<String, usize>,
    pub decisions_by_regime: BTreeMap<String, usize>,
    pub trades_by_regime: BTreeMap<String, usize>,
    pub net_return_by_regime: BTreeMap<String, f64>,
    pub max_drawdown_by_regime: BTreeMap<String, f64>,
    pub no_trade_rate_by_regime: BTreeMap<String, f64>,
    pub deny_rate_by_regime: BTreeMap<String, f64>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiskGovernorAggregate {
    pub total_denials: usize,
    pub total_cooldowns: usize,
    pub total_emergency_stops: usize,
    pub avoided_loss_count: usize,
    pub missed_gain_count: usize,
    pub defensive_value_total: f64,
    pub opportunity_cost_total: f64,
    pub most_common_denial_reasons: Vec<(String, usize)>,
    pub deny_rate_by_dataset: BTreeMap<String, f64>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelComparisonAggregate {
    pub compared_runs: usize,
    pub external_better_count: usize,
    pub baseline_better_count: usize,
    pub tie_count: usize,
    pub avg_delta_net_return_pct: f64,
    pub avg_delta_max_drawdown_pct: f64,
    pub avg_delta_calibration_brier: f64,
    pub external_failed_schema_count: usize,
    pub external_missing_prediction_count: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchExperimentReport {
    pub matrix_id: String,
    pub run_summaries: Vec<ExperimentRunSummary>,
    pub aggregate_benchmark: AggregateBenchmark,
    pub data_quality_summary: DataQualityAggregate,
    pub regime_summary: RegimeAggregate,
    pub risk_governor_summary: RiskGovernorAggregate,
    pub model_comparison_summary: ModelComparisonAggregate,
    pub persona_readiness_summary: super::readiness::PersonaReadinessSummary,
    pub expansion_readiness: super::readiness::ExpansionReadinessReport,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn summarize_run(
    dataset_id: &str,
    variant_id: &str,
    bundle: &ExperimentReportBundle,
) -> ExperimentRunSummary {
    let report = primary_report(bundle);
    let (
        total_decisions,
        executed_trades,
        denied_trades,
        no_trades,
        net_return_pct,
        max_drawdown_pct,
        profit_factor,
        risk_defensive_value,
    ) = if let Some(report) = report {
        (
            report.aggregate_metrics.decision_metrics.total_decisions,
            report.aggregate_metrics.trade_metrics.total_trades,
            report.aggregate_metrics.risk_metrics.denied_count,
            report.aggregate_metrics.decision_metrics.no_trade,
            report.aggregate_metrics.trade_metrics.net_return_pct,
            report.aggregate_metrics.trade_metrics.max_drawdown_pct,
            report.aggregate_metrics.trade_metrics.profit_factor,
            Some(report.aggregate_metrics.risk_metrics.defensive_value),
        )
    } else {
        (0, 0, 0, 0, 0.0, 0.0, None, None)
    };
    let status = if !bundle.errors.is_empty() {
        ExperimentRunStatus::Failed
    } else if matches!(
        bundle.data_quality_report.severity,
        DataQualitySeverity::Warning | DataQualitySeverity::Bad
    ) {
        ExperimentRunStatus::Warning
    } else {
        ExperimentRunStatus::Passed
    };
    ExperimentRunSummary {
        run_key: ExperimentRunKey {
            dataset_id: dataset_id.to_string(),
            variant_id: variant_id.to_string(),
            experiment_id: bundle.experiment_manifest.experiment_id.clone(),
        },
        status,
        manifest_summary: bundle.experiment_manifest.to_deterministic_string(),
        data_quality_score: bundle.data_quality_report.data_quality_score,
        data_quality_severity: bundle.data_quality_report.severity,
        total_decisions,
        executed_trades,
        denied_trades,
        no_trades,
        net_return_pct,
        max_drawdown_pct,
        profit_factor,
        calibration_brier: bundle
            .calibration_report
            .as_ref()
            .map(|report| report.brier_score),
        risk_defensive_value,
        external_better: bundle
            .model_comparison_report
            .as_ref()
            .map(|report| report.external_better),
        reason_codes: bundle.reason_codes.clone(),
    }
}

pub fn build_aggregate_benchmark(run_summaries: &[ExperimentRunSummary]) -> AggregateBenchmark {
    let active = run_summaries
        .iter()
        .filter(|summary| summary.status != ExperimentRunStatus::Skipped)
        .collect::<Vec<_>>();
    let mut net_returns = active
        .iter()
        .map(|summary| summary.net_return_pct)
        .collect::<Vec<_>>();
    net_returns.sort_by(|left, right| left.total_cmp(right));
    let profit_factors = active
        .iter()
        .filter_map(|summary| summary.profit_factor)
        .collect::<Vec<_>>();
    AggregateBenchmark {
        total_runs: run_summaries.len(),
        passed_runs: run_summaries
            .iter()
            .filter(|summary| summary.status == ExperimentRunStatus::Passed)
            .count(),
        failed_runs: run_summaries
            .iter()
            .filter(|summary| summary.status == ExperimentRunStatus::Failed)
            .count(),
        skipped_runs: run_summaries
            .iter()
            .filter(|summary| summary.status == ExperimentRunStatus::Skipped)
            .count(),
        baseline_runs: run_summaries
            .iter()
            .filter(|summary| summary.run_key.variant_id.contains("baseline"))
            .count(),
        external_runs: run_summaries
            .iter()
            .filter(|summary| {
                summary.run_key.variant_id.contains("external")
                    || summary.run_key.variant_id.contains("compare")
            })
            .count(),
        avg_net_return_pct: average(&net_returns),
        median_net_return_pct: median(&net_returns),
        worst_net_return_pct: net_returns.first().copied().unwrap_or(0.0),
        avg_max_drawdown_pct: average(
            &active
                .iter()
                .map(|summary| summary.max_drawdown_pct)
                .collect::<Vec<_>>(),
        ),
        worst_max_drawdown_pct: active
            .iter()
            .map(|summary| summary.max_drawdown_pct)
            .max_by(|left, right| left.total_cmp(right))
            .unwrap_or(0.0),
        avg_profit_factor: if profit_factors.is_empty() {
            None
        } else {
            Some(average(&profit_factors))
        },
        avg_no_trade_rate: average(
            &active
                .iter()
                .map(|summary| safe_ratio(summary.no_trades as f64, summary.total_decisions as f64))
                .collect::<Vec<_>>(),
        ),
        avg_denied_rate: average(
            &active
                .iter()
                .map(|summary| {
                    safe_ratio(summary.denied_trades as f64, summary.total_decisions as f64)
                })
                .collect::<Vec<_>>(),
        ),
        avg_data_quality_score: average(
            &active
                .iter()
                .map(|summary| summary.data_quality_score)
                .collect::<Vec<_>>(),
        ),
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

pub fn build_data_quality_aggregate(
    dataset_entries: &[(String, &ExperimentReportBundle)],
) -> DataQualityAggregate {
    let dataset_count = dataset_entries.len();
    let mut reason_counts = BTreeMap::<String, usize>::new();
    let mut gap_heavy = Vec::new();
    let mut duplicate_heavy = Vec::new();
    let mut invalid_ohlc = Vec::new();
    let mut worst = None::<(String, f64)>;
    let mut good_count = 0;
    let mut warning_count = 0;
    let mut bad_count = 0;
    let mut unusable_count = 0;
    for (dataset_id, bundle) in dataset_entries {
        let report = &bundle.data_quality_report;
        match report.severity {
            DataQualitySeverity::Good => good_count += 1,
            DataQualitySeverity::Warning => warning_count += 1,
            DataQualitySeverity::Bad => bad_count += 1,
            DataQualitySeverity::Unusable => unusable_count += 1,
        }
        if report.gap_count > 0 {
            gap_heavy.push(dataset_id.clone());
        }
        if report.duplicate_timestamp_count > 0 {
            duplicate_heavy.push(dataset_id.clone());
        }
        if report.ohlc_invariant_violation_count > 0 {
            invalid_ohlc.push(dataset_id.clone());
        }
        for reason in &report.reason_codes {
            *reason_counts.entry(format!("{reason:?}")).or_insert(0) += 1;
        }
        match &worst {
            Some((_, score)) if *score <= report.data_quality_score => {}
            _ => worst = Some((dataset_id.clone(), report.data_quality_score)),
        }
    }
    DataQualityAggregate {
        dataset_count,
        good_count,
        warning_count,
        bad_count,
        unusable_count,
        avg_data_quality_score: average(
            &dataset_entries
                .iter()
                .map(|(_, bundle)| bundle.data_quality_report.data_quality_score)
                .collect::<Vec<_>>(),
        ),
        worst_dataset_id: worst.map(|(dataset_id, _)| dataset_id),
        common_reason_codes: sort_counts(reason_counts),
        gap_heavy_datasets: gap_heavy,
        duplicate_heavy_datasets: duplicate_heavy,
        invalid_ohlc_datasets: invalid_ohlc,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

pub fn build_regime_aggregate(bundles: &[&ExperimentReportBundle]) -> RegimeAggregate {
    let mut counts = BTreeMap::new();
    let mut decisions = BTreeMap::new();
    let mut trades = BTreeMap::new();
    let mut net_returns = BTreeMap::new();
    let mut drawdowns = BTreeMap::new();
    let mut no_trade_rates = BTreeMap::<String, Vec<f64>>::new();
    let mut deny_rates = BTreeMap::<String, Vec<f64>>::new();
    for bundle in bundles {
        for regime in regime_metrics(bundle) {
            let key = format!("{:?}", regime.regime);
            *counts.entry(key.clone()).or_insert(0) += 1;
            *decisions.entry(key.clone()).or_insert(0) += regime.decision_metrics.total_decisions;
            *trades.entry(key.clone()).or_insert(0) += regime.trade_metrics.total_trades;
            *net_returns.entry(key.clone()).or_insert(0.0) += regime.trade_metrics.net_return_pct;
            let entry = drawdowns.entry(key.clone()).or_insert(0.0_f64);
            *entry = (*entry).max(regime.trade_metrics.max_drawdown_pct);
            no_trade_rates
                .entry(key.clone())
                .or_default()
                .push(safe_ratio(
                    regime.decision_metrics.no_trade as f64,
                    regime.decision_metrics.total_decisions as f64,
                ));
            deny_rates.entry(key).or_default().push(safe_ratio(
                regime.risk_metrics.denied_count as f64,
                regime.decision_metrics.total_decisions as f64,
            ));
        }
    }
    RegimeAggregate {
        counts_by_regime: counts,
        decisions_by_regime: decisions,
        trades_by_regime: trades,
        net_return_by_regime: net_returns,
        max_drawdown_by_regime: drawdowns,
        no_trade_rate_by_regime: no_trade_rates
            .into_iter()
            .map(|(key, values)| (key, average(&values)))
            .collect(),
        deny_rate_by_regime: deny_rates
            .into_iter()
            .map(|(key, values)| (key, average(&values)))
            .collect(),
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

pub fn build_risk_governor_aggregate(
    run_summaries: &[ExperimentRunSummary],
    bundles: &[(String, &ExperimentReportBundle)],
) -> RiskGovernorAggregate {
    let mut denial_reasons = BTreeMap::<String, usize>::new();
    let mut deny_rate_by_dataset = BTreeMap::new();
    let mut total_denials = 0usize;
    let mut total_cooldowns = 0usize;
    let mut total_emergency_stops = 0usize;
    let mut avoided_loss_count = 0usize;
    let mut missed_gain_count = 0usize;
    let mut defensive_value_total = 0.0;
    let mut opportunity_cost_total = 0.0;
    for (dataset_id, bundle) in bundles {
        let report = primary_report(bundle);
        if let Some(report) = report {
            total_denials += report.aggregate_metrics.risk_metrics.denied_count;
            total_cooldowns += report.aggregate_metrics.risk_metrics.cooldown_count;
            total_emergency_stops += report.aggregate_metrics.risk_metrics.emergency_stop_count;
            avoided_loss_count += report.aggregate_metrics.risk_metrics.avoided_loss_count;
            missed_gain_count += report.aggregate_metrics.risk_metrics.missed_gain_count;
            defensive_value_total += report.aggregate_metrics.risk_metrics.defensive_value;
            opportunity_cost_total += report.aggregate_metrics.risk_metrics.opportunity_cost;
            let total_decisions = report.aggregate_metrics.decision_metrics.total_decisions;
            deny_rate_by_dataset.insert(
                dataset_id.clone(),
                safe_ratio(
                    report.aggregate_metrics.risk_metrics.denied_count as f64,
                    total_decisions as f64,
                ),
            );
            for (reason, count) in &report.aggregate_metrics.decision_metrics.reason_code_counts {
                if reason.contains("GateBreached") || reason.contains("Denied") {
                    *denial_reasons.entry(reason.clone()).or_insert(0) += count;
                }
            }
        }
    }
    if denial_reasons.is_empty() {
        for summary in run_summaries {
            for reason in &summary.reason_codes {
                let name = format!("{reason:?}");
                if name.contains("GateBreached") || name.contains("Denied") {
                    *denial_reasons.entry(name).or_insert(0) += 1;
                }
            }
        }
    }
    RiskGovernorAggregate {
        total_denials,
        total_cooldowns,
        total_emergency_stops,
        avoided_loss_count,
        missed_gain_count,
        defensive_value_total,
        opportunity_cost_total,
        most_common_denial_reasons: sort_counts(denial_reasons),
        deny_rate_by_dataset,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

pub fn build_model_comparison_aggregate(
    bundles: &[&ExperimentReportBundle],
) -> ModelComparisonAggregate {
    let comparisons = bundles
        .iter()
        .filter_map(|bundle| {
            bundle
                .model_comparison_report
                .as_ref()
                .map(|report| (*bundle, report))
        })
        .collect::<Vec<_>>();
    let compared_runs = comparisons.len();
    let external_better_count = comparisons
        .iter()
        .filter(|(bundle, report)| {
            let schema_ok = bundle
                .prediction_validation_result
                .as_ref()
                .map(|result| result.valid)
                .unwrap_or(false);
            let calibration_ok = bundle
                .calibration_report
                .as_ref()
                .map(|report| report.brier_score <= 0.30)
                .unwrap_or(false);
            schema_ok
                && calibration_ok
                && report.external_better
                && report.delta_max_drawdown_pct <= 0.0
        })
        .count();
    let baseline_better_count = comparisons
        .iter()
        .filter(|(_, report)| !report.external_better && report.delta_net_return_pct < 0.0)
        .count();
    let tie_count = compared_runs.saturating_sub(external_better_count + baseline_better_count);
    ModelComparisonAggregate {
        compared_runs,
        external_better_count,
        baseline_better_count,
        tie_count,
        avg_delta_net_return_pct: average(
            &comparisons
                .iter()
                .map(|(_, report)| report.delta_net_return_pct)
                .collect::<Vec<_>>(),
        ),
        avg_delta_max_drawdown_pct: average(
            &comparisons
                .iter()
                .map(|(_, report)| report.delta_max_drawdown_pct)
                .collect::<Vec<_>>(),
        ),
        avg_delta_calibration_brier: average(
            &comparisons
                .iter()
                .map(|(bundle, _)| {
                    bundle
                        .calibration_report
                        .as_ref()
                        .map(|report| report.brier_score)
                        .unwrap_or(0.0)
                })
                .collect::<Vec<_>>(),
        ),
        external_failed_schema_count: bundles
            .iter()
            .filter(|bundle| {
                bundle
                    .prediction_validation_result
                    .as_ref()
                    .map(|result| !result.valid)
                    .unwrap_or(false)
            })
            .count(),
        external_missing_prediction_count: bundles
            .iter()
            .filter(|bundle| {
                bundle
                    .reason_codes
                    .contains(&ReasonCode::MissingPredictionRows)
            })
            .count(),
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

pub fn primary_report(bundle: &ExperimentReportBundle) -> Option<&crate::eval::WalkForwardReport> {
    bundle
        .external_walk_forward_report
        .as_ref()
        .or(bundle.baseline_walk_forward_report.as_ref())
}

pub fn regime_metrics(bundle: &ExperimentReportBundle) -> Vec<RegimeMetrics> {
    let mut map = BTreeMap::<Regime, RegimeMetrics>::new();
    if let Some(report) = primary_report(bundle) {
        for fold in &report.folds {
            for regime in &fold.regime_metrics {
                map.entry(regime.regime)
                    .and_modify(|existing| {
                        existing.trade_metrics.total_trades += regime.trade_metrics.total_trades;
                        existing.trade_metrics.net_return_pct +=
                            regime.trade_metrics.net_return_pct;
                        existing.trade_metrics.max_drawdown_pct = existing
                            .trade_metrics
                            .max_drawdown_pct
                            .max(regime.trade_metrics.max_drawdown_pct);
                        existing.decision_metrics.total_decisions +=
                            regime.decision_metrics.total_decisions;
                        existing.decision_metrics.no_trade += regime.decision_metrics.no_trade;
                        existing.risk_metrics.denied_count += regime.risk_metrics.denied_count;
                    })
                    .or_insert_with(|| regime.clone());
            }
        }
    }
    map.into_values().collect()
}

pub fn persona_metrics(bundle: &ExperimentReportBundle) -> Vec<PersonaFoldMetrics> {
    let mut map = BTreeMap::<String, PersonaFoldMetrics>::new();
    if let Some(report) = primary_report(bundle) {
        for fold in &report.folds {
            for persona in &fold.persona_metrics {
                map.entry(persona.persona_id.clone())
                    .and_modify(|existing| {
                        existing.selected_count += persona.selected_count;
                        existing.shadow_count += persona.shadow_count;
                        existing.supported_final_count += persona.supported_final_count;
                        existing.opposed_final_count += persona.opposed_final_count;
                        existing.forced_contrarian_count += persona.forced_contrarian_count;
                        existing.avg_contribution_score += persona.avg_contribution_score;
                        existing.net_attributed_return_pct += persona.net_attributed_return_pct;
                        existing.high_confidence_miss_count += persona.high_confidence_miss_count;
                    })
                    .or_insert_with(|| persona.clone());
            }
        }
    }
    map.into_values().collect()
}

fn sort_counts(counts: BTreeMap<String, usize>) -> Vec<(String, usize)> {
    let mut values = counts.into_iter().collect::<Vec<_>>();
    values.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    values
}

fn average(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

fn safe_ratio(numerator: f64, denominator: f64) -> f64 {
    if denominator <= 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}
