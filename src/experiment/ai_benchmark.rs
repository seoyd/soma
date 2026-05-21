use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::backtest::Timeframe;
use crate::core::ReasonCode;
use crate::data::{
    OfficialCollectionPlan, OfficialCollectionReport, OfficialCollectionRunner, ProviderKind,
};

use super::ai_usefulness::{
    AiSignalDecisionInputs, AiSignalUsefulnessReport, CalibrationSummary, ModelComparisonSummary,
    PerformanceSummary, RiskGovernorSummary, StorageBudgetSummary,
};
use super::config::{ExperimentConfig, ExperimentMode};
use super::model_gates::{
    ModelUsefulnessGateConfig, ModelUsefulnessGateInputs, ModelUsefulnessGateResult,
};
use super::official_coverage::OfficialDatasetCoverageReport;
use super::report_bundle::ExperimentReportBundle;
use super::risk_ai_interaction::RiskAiInteractionReport;
use super::runner::ExperimentRunner;
use super::storage_audit::BenchmarkStorageAudit;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialAiBenchmarkConfig {
    pub benchmark_id: String,
    #[serde(default)]
    pub official_collection_plan_path: Option<String>,
    #[serde(default)]
    pub official_collection_report_path: Option<String>,
    #[serde(default)]
    pub run_collection: bool,
    #[serde(default = "default_true")]
    pub run_dataset_export: bool,
    #[serde(default)]
    pub run_python_training: bool,
    #[serde(default)]
    pub run_external_prediction_eval: bool,
    #[serde(default = "default_true")]
    pub run_baseline_eval: bool,
    #[serde(default)]
    pub run_ablation_eval: bool,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default)]
    pub python_executable: Option<String>,
    #[serde(default)]
    pub train_script_path: Option<String>,
    #[serde(default)]
    pub existing_prediction_csv: Option<String>,
    #[serde(default = "default_true")]
    pub strict_schema_validation: bool,
    #[serde(default = "default_one")]
    pub min_official_ready_datasets: usize,
    #[serde(default = "default_twenty")]
    pub min_outcome_records: usize,
    #[serde(default = "default_twenty")]
    pub min_calibration_count: usize,
    #[serde(default = "default_one")]
    pub min_comparable_models: usize,
    #[serde(default = "default_drawdown")]
    pub max_allowed_drawdown_pct: f64,
    #[serde(default = "default_ece")]
    pub max_allowed_ece: f64,
    #[serde(default = "default_brier")]
    pub max_allowed_brier_score: f64,
    #[serde(default)]
    pub min_profit_factor: Option<f64>,
    #[serde(default)]
    pub min_net_return_pct: Option<f64>,
    #[serde(default)]
    pub allow_upbit_only: bool,
    #[serde(default = "default_true")]
    pub allow_equity_missing_auth: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for OfficialAiBenchmarkConfig {
    fn default() -> Self {
        Self {
            benchmark_id: "official-ai-benchmark".to_string(),
            official_collection_plan_path: None,
            official_collection_report_path: None,
            run_collection: false,
            run_dataset_export: true,
            run_python_training: false,
            run_external_prediction_eval: false,
            run_baseline_eval: true,
            run_ablation_eval: false,
            output_root: default_output_root(),
            python_executable: None,
            train_script_path: None,
            existing_prediction_csv: None,
            strict_schema_validation: true,
            min_official_ready_datasets: 1,
            min_outcome_records: 20,
            min_calibration_count: 20,
            min_comparable_models: 1,
            max_allowed_drawdown_pct: default_drawdown(),
            max_allowed_ece: default_ece(),
            max_allowed_brier_score: default_brier(),
            min_profit_factor: None,
            min_net_return_pct: None,
            allow_upbit_only: false,
            allow_equity_missing_auth: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OfficialAiBenchmarkConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let paths = [
            Some(self.output_root.as_str()),
            self.official_collection_plan_path.as_deref(),
            self.official_collection_report_path.as_deref(),
            self.python_executable.as_deref(),
            self.train_script_path.as_deref(),
            self.existing_prediction_csv.as_deref(),
        ];
        if paths
            .into_iter()
            .flatten()
            .any(|value| value.contains("://"))
        {
            vec![
                ReasonCode::LocalPathRejected,
                ReasonCode::OfficialAiBenchmarkConfigValidated,
            ]
        } else {
            vec![ReasonCode::OfficialAiBenchmarkConfigValidated]
        }
    }

    pub fn gate_config(&self) -> ModelUsefulnessGateConfig {
        ModelUsefulnessGateConfig {
            min_outcomes: self.min_outcome_records,
            max_drawdown_worsening_pct: self.max_allowed_drawdown_pct,
            max_ece: self.max_allowed_ece,
            max_brier_score: self.max_allowed_brier_score,
            min_profit_factor: self.min_profit_factor,
            require_not_worse_than_baseline: true,
            require_risk_stability: true,
            require_schema_valid: self.strict_schema_validation,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.benchmark_id)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialAiDatasetReport {
    pub entry_id: String,
    pub provider_kind: ProviderKind,
    pub symbol: String,
    pub timeframe: Timeframe,
    #[serde(default)]
    pub dataset_export_dir: Option<String>,
    #[serde(default)]
    pub baseline_output_dir: Option<String>,
    #[serde(default)]
    pub external_output_dir: Option<String>,
    pub baseline_total_trades: usize,
    #[serde(default)]
    pub external_total_trades: Option<usize>,
    #[serde(default)]
    pub schema_valid: Option<bool>,
    pub data_quality_score: f64,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialAiBenchmarkReport {
    pub benchmark_id: String,
    #[serde(default)]
    pub collection_report_path: Option<String>,
    pub coverage_report: OfficialDatasetCoverageReport,
    pub usefulness_gate_result: ModelUsefulnessGateResult,
    pub usefulness_report: AiSignalUsefulnessReport,
    pub storage_audit: BenchmarkStorageAudit,
    pub dataset_reports: Vec<OfficialAiDatasetReport>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl OfficialAiBenchmarkReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("benchmark_id={}", self.benchmark_id),
            format!(
                "collection_report_path={}",
                self.collection_report_path.as_deref().unwrap_or("")
            ),
            self.coverage_report.to_text(),
            self.usefulness_gate_result.to_text(),
            format!("status={:?}", self.usefulness_report.status),
            format!("recommendation={:?}", self.usefulness_report.recommendation),
            format!(
                "official_dataset_count={}",
                self.usefulness_report.official_dataset_count
            ),
            format!(
                "total_outcome_records={}",
                self.usefulness_report.total_outcome_records
            ),
            self.storage_audit.to_text(),
            format!("warnings={}", self.warnings.join(" | ")),
        ];
        for dataset in &self.dataset_reports {
            lines.push(format!(
                "dataset={};provider={:?};symbol={};baseline_trades={};external_trades={};schema_valid={};quality={:.6}",
                dataset.entry_id,
                dataset.provider_kind,
                dataset.symbol,
                dataset.baseline_total_trades,
                dataset
                    .external_total_trades
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                dataset
                    .schema_valid
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                dataset.data_quality_score
            ));
        }
        lines.join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_ai_benchmark_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_ai_benchmark_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("ai_signal_usefulness_report.md"),
            self.usefulness_report.to_markdown(),
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OfficialAiBenchmarkRunner {
    pub collection_runner: OfficialCollectionRunner,
    pub experiment_runner: ExperimentRunner,
}

impl OfficialAiBenchmarkRunner {
    pub fn run(&self, config: &OfficialAiBenchmarkConfig) -> OfficialAiBenchmarkReport {
        let mut warnings = Vec::new();
        let mut reason_codes = config.validate_local_paths();
        let benchmark_dir = config.output_dir();
        let _ = fs::create_dir_all(&benchmark_dir);

        let invalid_paths = reason_codes.contains(&ReasonCode::LocalPathRejected);
        if invalid_paths {
            let report = empty_benchmark_report(
                config,
                None,
                warnings,
                vec![
                    ReasonCode::LocalPathRejected,
                    ReasonCode::OfficialAiBenchmarkConfigValidated,
                ],
            );
            let _ = report.write_to_dir(&benchmark_dir);
            return report;
        }

        let (collection_report, collection_report_path) =
            self.load_or_run_collection(config, &mut warnings, &mut reason_codes);
        let coverage_report =
            OfficialDatasetCoverageReport::from_collection_report(&collection_report);
        let ready_entries = collection_report
            .entry_reports
            .iter()
            .filter(|entry| entry.ready_for_evidence)
            .cloned()
            .collect::<Vec<_>>();
        let official_ready_entries = ready_entries
            .iter()
            .filter(|entry| entry.provider_kind != ProviderKind::MockFixture)
            .cloned()
            .collect::<Vec<_>>();
        let mut dataset_reports = Vec::new();
        let mut baseline_bundles = Vec::new();
        let mut external_bundles = Vec::new();
        let mut risk_reports = Vec::new();
        let mut bundle_dirs = Vec::new();

        if config.run_ablation_eval {
            warnings.push(
                "run_ablation_eval requested; benchmark runner keeps ablation deferred to the existing ablation workflow".to_string(),
            );
            reason_codes.push(ReasonCode::OfficialAiBenchmarkAblationDeferred);
        }

        for entry in official_ready_entries {
            if let Some(dataset_report) = self.run_entry(
                config,
                &benchmark_dir,
                &entry,
                &mut bundle_dirs,
                &mut baseline_bundles,
                &mut external_bundles,
                &mut risk_reports,
            ) {
                dataset_reports.push(dataset_report);
            } else {
                warnings.push(format!(
                    "skipped benchmark entry {} due to missing canonical path",
                    entry.entry_id
                ));
            }
        }

        let baseline_summary = aggregate_performance_summary(
            &baseline_bundles
                .iter()
                .filter_map(|bundle| bundle.baseline_walk_forward_report.as_ref())
                .collect::<Vec<_>>(),
        );
        let external_reports = external_bundles
            .iter()
            .filter_map(|bundle| bundle.external_walk_forward_report.as_ref())
            .collect::<Vec<_>>();
        let external_summary = (!external_reports.is_empty())
            .then(|| aggregate_performance_summary(&external_reports));
        let comparison_reports = external_bundles
            .iter()
            .filter_map(|bundle| bundle.model_comparison_report.as_ref())
            .collect::<Vec<_>>();
        let comparison_summary = (!comparison_reports.is_empty())
            .then(|| aggregate_model_comparison_summary(&comparison_reports));
        let calibration_summary =
            aggregate_calibration_summary(&baseline_bundles, &external_bundles);
        let risk_summary = aggregate_risk_summary(&risk_reports);
        let storage_audit =
            BenchmarkStorageAudit::build(&collection_report, &bundle_dirs, &benchmark_dir);
        let external_evaluation_completed = !external_bundles.is_empty()
            && external_bundles.iter().all(|bundle| {
                bundle
                    .prediction_validation_result
                    .as_ref()
                    .map(|value| value.valid)
                    .unwrap_or(bundle.external_walk_forward_report.is_some())
            })
            && external_summary.is_some();
        let gate_result = ModelUsefulnessGateResult::evaluate(
            &config.gate_config(),
            &ModelUsefulnessGateInputs {
                schema_valid: external_bundles.iter().all(|bundle| {
                    bundle
                        .prediction_validation_result
                        .as_ref()
                        .map(|value| value.valid)
                        .unwrap_or(true)
                }),
                outcome_count: external_summary
                    .as_ref()
                    .map(|summary| summary.total_trades)
                    .unwrap_or(baseline_summary.total_trades),
                calibration_count: calibration_summary.total_count,
                brier_score: calibration_summary.avg_brier_score,
                expected_calibration_error: calibration_summary.avg_expected_calibration_error,
                selected_profit_factor: external_summary
                    .as_ref()
                    .map(|summary| summary.avg_profit_factor)
                    .filter(|value| *value > 0.0)
                    .or_else(|| {
                        (baseline_summary.avg_profit_factor > 0.0)
                            .then_some(baseline_summary.avg_profit_factor)
                    }),
                delta_max_drawdown_pct: comparison_summary
                    .as_ref()
                    .map(|summary| summary.avg_delta_max_drawdown_pct),
                delta_net_return_pct: comparison_summary
                    .as_ref()
                    .map(|summary| summary.avg_delta_net_return_pct),
                denial_rate: risk_summary.denial_rate,
                approval_rate: risk_summary.approval_rate,
                emergency_stop_count: risk_summary.emergency_stop_count,
                leakage_detected: has_leakage(&baseline_bundles) || has_leakage(&external_bundles),
                data_quality_score: average(
                    &dataset_reports
                        .iter()
                        .map(|report| report.data_quality_score)
                        .collect::<Vec<_>>(),
                ),
                budget_exceeded: storage_audit.budget_exceeded,
            },
        );
        let usefulness_report = AiSignalUsefulnessReport::decide(
            &coverage_report,
            &gate_result,
            AiSignalDecisionInputs {
                official_dataset_count: dataset_reports.len(),
                total_outcome_records: external_summary
                    .as_ref()
                    .map(|summary| summary.total_trades)
                    .unwrap_or(baseline_summary.total_trades),
                baseline_summary,
                external_summary,
                calibration_summary,
                risk_governor_summary: risk_summary,
                model_comparison_summary: comparison_summary.clone(),
                storage_budget_summary: StorageBudgetSummary {
                    collection_bytes: storage_audit.collection_bytes,
                    dataset_export_bytes: storage_audit.dataset_export_bytes,
                    prediction_bytes: storage_audit.prediction_bytes,
                    report_bytes: storage_audit.report_bytes,
                    budget_exceeded: storage_audit.budget_exceeded,
                },
                has_external_evaluation: external_evaluation_completed,
                comparison_external_better: comparison_summary.as_ref().is_some_and(|summary| {
                    external_evaluation_completed && summary.external_better_count > 0
                }),
                missing_auth: !coverage_report.missing_auth_providers.is_empty(),
                non_official_ready_entries: coverage_report.non_official_ready_entries,
                allow_upbit_only: config.allow_upbit_only,
                allow_equity_missing_auth: config.allow_equity_missing_auth,
                min_official_ready_datasets: config.min_official_ready_datasets,
            },
        );
        reason_codes.push(ReasonCode::OfficialAiBenchmarkRan);
        let report = OfficialAiBenchmarkReport {
            benchmark_id: config.benchmark_id.clone(),
            collection_report_path,
            coverage_report,
            usefulness_gate_result: gate_result,
            usefulness_report,
            storage_audit,
            dataset_reports,
            warnings,
            reason_codes: dedupe_reasons(reason_codes),
        };
        let _ = report.write_to_dir(&benchmark_dir);
        report
    }

    #[allow(clippy::too_many_arguments)]
    fn run_entry(
        &self,
        config: &OfficialAiBenchmarkConfig,
        benchmark_dir: &Path,
        entry: &crate::data::OfficialCollectionEntryReport,
        bundle_dirs: &mut Vec<PathBuf>,
        baseline_bundles: &mut Vec<ExperimentReportBundle>,
        external_bundles: &mut Vec<ExperimentReportBundle>,
        risk_reports: &mut Vec<RiskAiInteractionReport>,
    ) -> Option<OfficialAiDatasetReport> {
        let canonical_path = entry.canonical_csv_path.as_deref().map(PathBuf::from)?;
        if !canonical_path.exists() {
            return None;
        }

        let base_dir = benchmark_dir.join(&entry.entry_id);
        let mut warnings = Vec::new();
        let mut reason_codes = vec![ReasonCode::OfficialAiBenchmarkRan];
        let mut dataset_export_dir = None;
        let mut baseline_output_dir = None;
        let mut external_output_dir = None;
        let mut baseline_total_trades = 0usize;
        let mut external_total_trades = None;
        let mut schema_valid = None;
        let mut data_quality_score = 0.0;

        if config.run_dataset_export {
            let export_config = benchmark_experiment_config(
                &format!("{}-dataset", entry.entry_id),
                &entry.symbol,
                &canonical_path,
                entry.timeframe,
                &base_dir.join("dataset_export"),
                ExperimentMode::DatasetExportOnly,
                config,
            );
            let bundle = self.experiment_runner.run(&export_config);
            dataset_export_dir = Some(export_config.output_bundle_dir().display().to_string());
            bundle_dirs.push(export_config.output_bundle_dir());
            data_quality_score = bundle.data_quality_report.data_quality_score;
        }
        if config.run_baseline_eval {
            let baseline_config = benchmark_experiment_config(
                &format!("{}-baseline", entry.entry_id),
                &entry.symbol,
                &canonical_path,
                entry.timeframe,
                &base_dir.join("baseline"),
                ExperimentMode::BaselineOnly,
                config,
            );
            let bundle = self.experiment_runner.run(&baseline_config);
            baseline_total_trades = bundle
                .baseline_walk_forward_report
                .as_ref()
                .map(|report| report.aggregate_metrics.trade_metrics.total_trades)
                .unwrap_or(0);
            data_quality_score = bundle.data_quality_report.data_quality_score;
            if let Some(report) = &bundle.baseline_walk_forward_report {
                risk_reports.push(RiskAiInteractionReport::from_walk_forward_report(
                    format!("{}-baseline", entry.entry_id),
                    report,
                ));
            }
            baseline_output_dir = Some(baseline_config.output_bundle_dir().display().to_string());
            bundle_dirs.push(baseline_config.output_bundle_dir());
            baseline_bundles.push(bundle);
        }

        let wants_external = config.run_external_prediction_eval
            || config.run_python_training
            || config.existing_prediction_csv.is_some();
        if wants_external {
            let external_mode = if config.run_baseline_eval {
                ExperimentMode::TrainAndCompare
            } else {
                ExperimentMode::ExternalPredictionOnly
            };
            let external_config = benchmark_experiment_config(
                &format!("{}-external", entry.entry_id),
                &entry.symbol,
                &canonical_path,
                entry.timeframe,
                &base_dir.join("external"),
                external_mode,
                config,
            );
            let bundle = self.experiment_runner.run(&external_config);
            if let Some(report) = &bundle.external_walk_forward_report {
                external_total_trades = Some(report.aggregate_metrics.trade_metrics.total_trades);
                risk_reports.push(RiskAiInteractionReport::from_walk_forward_report(
                    format!("{}-external", entry.entry_id),
                    report,
                ));
            } else {
                warnings
                    .push("external evaluation did not produce a walk-forward report".to_string());
                reason_codes.push(ReasonCode::OfficialAiBenchmarkExternalRejected);
            }
            schema_valid = bundle
                .prediction_validation_result
                .as_ref()
                .map(|value| value.valid);
            external_output_dir = Some(external_config.output_bundle_dir().display().to_string());
            bundle_dirs.push(external_config.output_bundle_dir());
            external_bundles.push(bundle);
            reason_codes.push(ReasonCode::OfficialAiBenchmarkExternalEvaluated);
        } else if config.run_baseline_eval {
            reason_codes.push(ReasonCode::OfficialAiBenchmarkBaselineOnly);
        }

        Some(OfficialAiDatasetReport {
            entry_id: entry.entry_id.clone(),
            provider_kind: entry.provider_kind,
            symbol: entry.symbol.clone(),
            timeframe: entry.timeframe,
            dataset_export_dir,
            baseline_output_dir,
            external_output_dir,
            baseline_total_trades,
            external_total_trades,
            schema_valid,
            data_quality_score,
            warnings,
            reason_codes: dedupe_reasons(reason_codes),
        })
    }

    fn load_or_run_collection(
        &self,
        config: &OfficialAiBenchmarkConfig,
        warnings: &mut Vec<String>,
        reason_codes: &mut Vec<ReasonCode>,
    ) -> (OfficialCollectionReport, Option<String>) {
        if config.run_collection {
            let Some(path) = config.official_collection_plan_path.as_deref() else {
                warnings.push(
                    "run_collection=true but no official_collection_plan_path provided".to_string(),
                );
                return (empty_collection_report(&config.benchmark_id), None);
            };
            match OfficialCollectionPlan::from_toml_path(Path::new(path)) {
                Ok(plan) => {
                    let report = self.collection_runner.run_plan(&plan);
                    let report_path = Path::new(&plan.output_root)
                        .join(&plan.plan_id)
                        .join("official_collection_report.json");
                    reason_codes.push(ReasonCode::OfficialCollectionRan);
                    (report, Some(report_path.display().to_string()))
                }
                Err(err) => {
                    warnings.push(format!("failed to load official collection plan: {err}"));
                    (empty_collection_report(&config.benchmark_id), None)
                }
            }
        } else if let Some(path) = config.official_collection_report_path.as_deref() {
            match OfficialCollectionReport::from_json_path(Path::new(path)) {
                Ok(report) => (report, Some(path.to_string())),
                Err(err) => {
                    warnings.push(format!("failed to load official collection report: {err}"));
                    (empty_collection_report(&config.benchmark_id), None)
                }
            }
        } else {
            warnings.push("no official collection input configured".to_string());
            (empty_collection_report(&config.benchmark_id), None)
        }
    }
}

fn benchmark_experiment_config(
    experiment_id: &str,
    symbol: &str,
    canonical_path: &Path,
    timeframe: Timeframe,
    output_dir: &Path,
    mode: ExperimentMode,
    benchmark_config: &OfficialAiBenchmarkConfig,
) -> ExperimentConfig {
    let mut config = match mode {
        ExperimentMode::DatasetExportOnly => ExperimentConfig::dataset_export_only(
            experiment_id,
            symbol,
            canonical_path.display().to_string(),
            timeframe,
            output_dir.display().to_string(),
        ),
        _ => ExperimentConfig::baseline_only(
            experiment_id,
            symbol,
            canonical_path.display().to_string(),
            timeframe,
            output_dir.display().to_string(),
        ),
    };
    config.mode = mode;
    config.walk_forward_config.train_window_bars = 5;
    config.walk_forward_config.validation_window_bars = Some(2);
    config.walk_forward_config.test_window_bars = 3;
    config.walk_forward_config.step_bars = 2;
    config.walk_forward_config.embargo_bars = 0;
    config.walk_forward_config.min_train_bars = 5;
    config.walk_forward_config.max_folds = Some(2);
    config.feature_config.min_required_bars = 5;
    config.triple_barrier_config.horizon_bars = 2;
    config.run_python_training = benchmark_config.run_python_training;
    config.python_executable = benchmark_config.python_executable.clone();
    config.training_script_path = benchmark_config.train_script_path.clone();
    config.prediction_csv_path = benchmark_config.existing_prediction_csv.clone();
    config.strict_schema_validation = benchmark_config.strict_schema_validation;
    config.reason_codes = vec![ReasonCode::DeterministicPath];
    config
}

fn aggregate_performance_summary(
    reports: &[&crate::eval::WalkForwardReport],
) -> PerformanceSummary {
    PerformanceSummary {
        dataset_count: reports.len(),
        total_trades: reports
            .iter()
            .map(|report| report.aggregate_metrics.trade_metrics.total_trades)
            .sum(),
        avg_net_return_pct: average(
            &reports
                .iter()
                .map(|report| report.aggregate_metrics.trade_metrics.net_return_pct)
                .collect::<Vec<_>>(),
        ),
        avg_profit_factor: average(
            &reports
                .iter()
                .map(|report| {
                    report
                        .aggregate_metrics
                        .trade_metrics
                        .profit_factor
                        .unwrap_or(0.0)
                })
                .collect::<Vec<_>>(),
        ),
        avg_max_drawdown_pct: average(
            &reports
                .iter()
                .map(|report| report.aggregate_metrics.trade_metrics.max_drawdown_pct)
                .collect::<Vec<_>>(),
        ),
    }
}

fn aggregate_model_comparison_summary(
    reports: &[&crate::model::ModelComparisonReport],
) -> ModelComparisonSummary {
    ModelComparisonSummary {
        compared_datasets: reports.len(),
        external_better_count: reports
            .iter()
            .filter(|report| report.external_better)
            .count(),
        avg_delta_net_return_pct: average(
            &reports
                .iter()
                .map(|report| report.delta_net_return_pct)
                .collect::<Vec<_>>(),
        ),
        avg_delta_max_drawdown_pct: average(
            &reports
                .iter()
                .map(|report| report.delta_max_drawdown_pct)
                .collect::<Vec<_>>(),
        ),
        avg_delta_profit_factor: average(
            &reports
                .iter()
                .map(|report| report.delta_profit_factor)
                .collect::<Vec<_>>(),
        ),
    }
}

fn aggregate_calibration_summary(
    baseline_bundles: &[ExperimentReportBundle],
    external_bundles: &[ExperimentReportBundle],
) -> CalibrationSummary {
    let external_calibrations = external_bundles
        .iter()
        .filter_map(|bundle| bundle.calibration_report.as_ref())
        .collect::<Vec<_>>();
    if !external_calibrations.is_empty() {
        return CalibrationSummary {
            total_count: external_calibrations
                .iter()
                .map(|report| report.total_count)
                .sum(),
            avg_brier_score: average(
                &external_calibrations
                    .iter()
                    .map(|report| report.brier_score)
                    .collect::<Vec<_>>(),
            ),
            avg_expected_calibration_error: average(
                &external_calibrations
                    .iter()
                    .map(|report| report.expected_calibration_error)
                    .collect::<Vec<_>>(),
            ),
            acceptable: true,
        };
    }
    let baseline_reports = baseline_bundles
        .iter()
        .filter_map(|bundle| bundle.baseline_walk_forward_report.as_ref())
        .collect::<Vec<_>>();
    CalibrationSummary {
        total_count: baseline_reports
            .iter()
            .map(|report| report.aggregate_metrics.trade_metrics.total_trades)
            .sum(),
        avg_brier_score: average(
            &baseline_reports
                .iter()
                .map(|report| report.aggregate_metrics.calibration_metrics.brier_score)
                .collect::<Vec<_>>(),
        ),
        avg_expected_calibration_error: average(
            &baseline_reports
                .iter()
                .map(|report| {
                    report
                        .aggregate_metrics
                        .calibration_metrics
                        .expected_calibration_error
                        .unwrap_or(0.0)
                })
                .collect::<Vec<_>>(),
        ),
        acceptable: true,
    }
}

fn aggregate_risk_summary(reports: &[RiskAiInteractionReport]) -> RiskGovernorSummary {
    RiskGovernorSummary {
        total_signals: reports.iter().map(|report| report.total_signals).sum(),
        denied_by_risk: reports.iter().map(|report| report.denied_by_risk).sum(),
        denial_rate: average(
            &reports
                .iter()
                .map(|report| report.denial_rate)
                .collect::<Vec<_>>(),
        ),
        approval_rate: average(
            &reports
                .iter()
                .map(|report| report.approval_rate)
                .collect::<Vec<_>>(),
        ),
        emergency_stop_count: reports
            .iter()
            .map(|report| report.emergency_stop_count)
            .sum(),
        cooldown_count: reports.iter().map(|report| report.cooldown_count).sum(),
        defensive_value: reports.iter().map(|report| report.defensive_value).sum(),
        opportunity_cost: reports.iter().map(|report| report.opportunity_cost).sum(),
        stable: reports.iter().all(|report| report.warnings.is_empty()),
    }
}

fn has_leakage(bundles: &[ExperimentReportBundle]) -> bool {
    bundles.iter().any(|bundle| {
        bundle
            .baseline_walk_forward_report
            .as_ref()
            .is_some_and(|report| {
                report
                    .folds
                    .iter()
                    .any(|fold| fold.leakage_report.has_leakage)
            })
            || bundle
                .external_walk_forward_report
                .as_ref()
                .is_some_and(|report| {
                    report
                        .folds
                        .iter()
                        .any(|fold| fold.leakage_report.has_leakage)
                })
    })
}

fn empty_collection_report(plan_id: &str) -> OfficialCollectionReport {
    OfficialCollectionReport {
        plan_id: plan_id.to_string(),
        entry_reports: Vec::new(),
        storage_budget_report: crate::data::StorageBudgetReport::default(),
        ready_entries_count: 0,
        skipped_entries_count: 0,
        failed_entries_count: 0,
        official_api_collected_count: 0,
        reason_codes: vec![ReasonCode::MissingFile],
    }
}

fn empty_benchmark_report(
    config: &OfficialAiBenchmarkConfig,
    collection_report_path: Option<String>,
    warnings: Vec<String>,
    reason_codes: Vec<ReasonCode>,
) -> OfficialAiBenchmarkReport {
    let coverage_report = OfficialDatasetCoverageReport::from_collection_report(
        &empty_collection_report(&config.benchmark_id),
    );
    let storage_audit = BenchmarkStorageAudit::build(
        &empty_collection_report(&config.benchmark_id),
        &[],
        &config.output_dir(),
    );
    let gate_result = ModelUsefulnessGateResult::evaluate(
        &config.gate_config(),
        &ModelUsefulnessGateInputs {
            schema_valid: false,
            outcome_count: 0,
            calibration_count: 0,
            brier_score: 0.0,
            expected_calibration_error: 0.0,
            selected_profit_factor: None,
            delta_max_drawdown_pct: None,
            delta_net_return_pct: None,
            denial_rate: 0.0,
            approval_rate: 0.0,
            emergency_stop_count: 0,
            leakage_detected: false,
            data_quality_score: 0.0,
            budget_exceeded: false,
        },
    );
    let usefulness_report = AiSignalUsefulnessReport::decide(
        &coverage_report,
        &gate_result,
        AiSignalDecisionInputs {
            official_dataset_count: 0,
            total_outcome_records: 0,
            baseline_summary: PerformanceSummary::default(),
            external_summary: None,
            calibration_summary: CalibrationSummary::default(),
            risk_governor_summary: RiskGovernorSummary::default(),
            model_comparison_summary: None,
            storage_budget_summary: StorageBudgetSummary {
                collection_bytes: 0,
                dataset_export_bytes: 0,
                prediction_bytes: 0,
                report_bytes: 0,
                budget_exceeded: false,
            },
            has_external_evaluation: false,
            comparison_external_better: false,
            missing_auth: false,
            non_official_ready_entries: 0,
            allow_upbit_only: config.allow_upbit_only,
            allow_equity_missing_auth: config.allow_equity_missing_auth,
            min_official_ready_datasets: config.min_official_ready_datasets,
        },
    );
    OfficialAiBenchmarkReport {
        benchmark_id: config.benchmark_id.clone(),
        collection_report_path,
        coverage_report,
        usefulness_gate_result: gate_result,
        usefulness_report,
        storage_audit,
        dataset_reports: Vec::new(),
        warnings,
        reason_codes,
    }
}

fn average(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn dedupe_reasons(values: Vec<ReasonCode>) -> Vec<ReasonCode> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    deduped
}

fn default_true() -> bool {
    true
}

fn default_output_root() -> String {
    "target/soma_official_ai_benchmark".to_string()
}

fn default_one() -> usize {
    1
}

fn default_twenty() -> usize {
    20
}

fn default_drawdown() -> f64 {
    0.05
}

fn default_ece() -> f64 {
    0.10
}

fn default_brier() -> f64 {
    0.30
}
