use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::DataQualitySeverity;

use super::aggregate::{BatchExperimentReport, ExperimentRunStatus};
use super::batch::BatchExperimentRunner;
use super::diff::{CampaignDiffReport, build_campaign_diff_report};
use super::evidence::{EvidenceStore, EvidenceStoreConfig};
use super::matrix::ExperimentMatrixConfig;
use super::readiness::{
    CampaignExpansionReadinessEvidence, CampaignExpansionReadinessReport,
    ExpansionReadinessDecision, build_campaign_expansion_readiness_report,
};
use super::regression::{RegressionGuardConfig, RegressionGuardResult, evaluate_regression_guard};
use super::render::{
    campaign_summary_to_markdown_table, campaign_summary_to_text, diff_report_to_text,
    readiness_report_to_text,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResearchCampaignConfig {
    pub campaign_id: String,
    pub description: Option<String>,
    pub matrix_config_paths: Vec<String>,
    #[serde(default)]
    pub embedded_matrices: Vec<ExperimentMatrixConfig>,
    pub output_root: String,
    pub evidence_store_path: String,
    pub run_id: Option<String>,
    pub continue_on_failure: bool,
    pub require_all_matrices_pass: bool,
    pub compare_against_campaign_id: Option<String>,
    pub compare_against_report_path: Option<String>,
    pub min_usable_datasets: usize,
    pub min_total_outcome_records: usize,
    pub min_regime_coverage_count: usize,
    pub min_passed_runs: usize,
    pub min_data_quality_score: f64,
    pub max_allowed_drawdown_regression_pct: f64,
    pub max_allowed_calibration_regression: f64,
    pub max_allowed_risk_governor_instability: f64,
    pub allow_persona_expansion_recommendation: bool,
    #[serde(default)]
    pub created_at_ms: Option<u64>,
    #[serde(default = "default_true")]
    pub allow_evidence_overwrite: bool,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for ResearchCampaignConfig {
    fn default() -> Self {
        Self {
            campaign_id: "campaign".to_string(),
            description: None,
            matrix_config_paths: Vec::new(),
            embedded_matrices: Vec::new(),
            output_root: "target/soma_campaigns".to_string(),
            evidence_store_path: "target/soma_evidence".to_string(),
            run_id: None,
            continue_on_failure: true,
            require_all_matrices_pass: false,
            compare_against_campaign_id: None,
            compare_against_report_path: None,
            min_usable_datasets: 2,
            min_total_outcome_records: 20,
            min_regime_coverage_count: 2,
            min_passed_runs: 2,
            min_data_quality_score: 0.80,
            max_allowed_drawdown_regression_pct: 0.02,
            max_allowed_calibration_regression: 0.02,
            max_allowed_risk_governor_instability: 0.15,
            allow_persona_expansion_recommendation: false,
            created_at_ms: None,
            allow_evidence_overwrite: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl ResearchCampaignConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&contents).map_err(|err| err.to_string())
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let mut remote = self.output_root.contains("://")
            || self.evidence_store_path.contains("://")
            || self
                .matrix_config_paths
                .iter()
                .any(|path| path.contains("://"))
            || self
                .compare_against_report_path
                .as_deref()
                .is_some_and(|path| path.contains("://"));
        remote |= self
            .embedded_matrices
            .iter()
            .flat_map(|matrix| matrix.validate_local_paths())
            .next()
            .is_some();
        if remote {
            vec![
                ReasonCode::LocalPathRejected,
                ReasonCode::CampaignConfigInvalid,
            ]
        } else {
            Vec::new()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CampaignMatrixStatus {
    Passed,
    Warning,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CampaignMatrixResult {
    pub matrix_id: String,
    pub source: String,
    pub status: CampaignMatrixStatus,
    pub report: Option<BatchExperimentReport>,
    pub reason_codes: Vec<ReasonCode>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CampaignAggregate {
    pub campaign_id: String,
    pub matrix_count: usize,
    pub total_runs: usize,
    pub passed_runs: usize,
    pub failed_runs: usize,
    pub skipped_runs: usize,
    pub usable_dataset_count: usize,
    pub total_dataset_count: usize,
    pub total_outcome_records: usize,
    pub total_executed_trades: usize,
    pub total_no_trades: usize,
    pub total_denials: usize,
    pub average_data_quality_score: f64,
    pub worst_data_quality_score: f64,
    pub average_net_return_pct: f64,
    pub median_net_return_pct: f64,
    pub worst_net_return_pct: f64,
    pub average_max_drawdown_pct: f64,
    pub worst_max_drawdown_pct: f64,
    pub average_profit_factor: Option<f64>,
    pub average_calibration_brier: Option<f64>,
    pub regime_coverage_count: usize,
    pub unknown_regime_rate: f64,
    pub panic_regime_rate: f64,
    pub risk_defensive_value_total: f64,
    pub risk_opportunity_cost_total: f64,
    pub persona_redundancy_warning_count: usize,
    pub external_model_validated_count: usize,
    pub reason_codes: Vec<ReasonCode>,
}

impl CampaignAggregate {
    pub fn to_markdown_table_string(&self) -> String {
        [
            "| metric | value |".to_string(),
            "| --- | ---: |".to_string(),
            format!("| matrix_count | {} |", self.matrix_count),
            format!("| total_runs | {} |", self.total_runs),
            format!("| passed_runs | {} |", self.passed_runs),
            format!("| failed_runs | {} |", self.failed_runs),
            format!("| skipped_runs | {} |", self.skipped_runs),
            format!("| usable_dataset_count | {} |", self.usable_dataset_count),
            format!("| total_dataset_count | {} |", self.total_dataset_count),
            format!("| total_outcome_records | {} |", self.total_outcome_records),
            format!(
                "| average_data_quality_score | {:.8} |",
                self.average_data_quality_score
            ),
            format!(
                "| average_net_return_pct | {:.8} |",
                self.average_net_return_pct
            ),
            format!(
                "| worst_max_drawdown_pct | {:.8} |",
                self.worst_max_drawdown_pct
            ),
            format!("| regime_coverage_count | {} |", self.regime_coverage_count),
            format!(
                "| risk_defensive_value_total | {:.8} |",
                self.risk_defensive_value_total
            ),
        ]
        .join("\n")
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResearchCampaignReport {
    pub campaign_id: String,
    pub description: Option<String>,
    pub matrix_results: Vec<CampaignMatrixResult>,
    pub aggregate: CampaignAggregate,
    pub diff_report: CampaignDiffReport,
    pub regression_guard: RegressionGuardResult,
    pub readiness_report: CampaignExpansionReadinessReport,
    pub reason_codes: Vec<ReasonCode>,
    pub errors: Vec<String>,
}

impl ResearchCampaignReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&contents).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("campaign_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("campaign_summary.txt"),
            campaign_summary_to_text(self),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("campaign_summary.md"),
            campaign_summary_to_markdown_table(&self.aggregate),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("campaign_diff.txt"),
            diff_report_to_text(&self.diff_report),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("readiness_report.txt"),
            readiness_report_to_text(&self.readiness_report),
        )
        .map_err(|err| err.to_string())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResearchCampaignRunner {
    pub batch_runner: BatchExperimentRunner,
    pub evidence_store: EvidenceStore,
}

#[derive(Clone, Debug)]
struct LoadedMatrixConfig {
    source: String,
    config: ExperimentMatrixConfig,
}

impl ResearchCampaignRunner {
    pub fn run_campaign(&self, config: &ResearchCampaignConfig) -> ResearchCampaignReport {
        let invalid = config.validate_local_paths();
        if !invalid.is_empty() {
            let aggregate = build_campaign_aggregate(&config.campaign_id, &[]);
            return minimal_report(
                config,
                Vec::new(),
                aggregate.clone(),
                build_campaign_diff_report(
                    &aggregate,
                    None,
                    config.compare_against_campaign_id.as_deref(),
                ),
                invalid,
                vec!["campaign config contains remote URL-like path".to_string()],
            );
        }

        let (loaded_matrices, mut matrix_results, matrix_inputs) = self.load_matrices(config);
        for loaded in loaded_matrices {
            let report = self.batch_runner.run_matrix(&loaded.config);
            let failed = campaign_matrix_status(&report) == CampaignMatrixStatus::Failed;
            matrix_results.push(CampaignMatrixResult {
                matrix_id: report.matrix_id.clone(),
                source: loaded.source,
                status: campaign_matrix_status(&report),
                reason_codes: report.reason_codes.clone(),
                report: Some(report),
                error: None,
            });
            if failed && !config.continue_on_failure {
                break;
            }
        }

        let mut reason_codes = Vec::new();
        if config.require_all_matrices_pass
            && matrix_results
                .iter()
                .any(|result| result.status == CampaignMatrixStatus::Failed)
        {
            reason_codes.push(ReasonCode::CampaignRequireAllPassFailed);
        }
        let errors = matrix_results
            .iter()
            .filter_map(|result| result.error.clone())
            .collect::<Vec<_>>();
        let aggregate = build_campaign_aggregate(&config.campaign_id, &matrix_results);
        let previous_report = self.load_previous_report(config);
        let diff_report = build_campaign_diff_report(
            &aggregate,
            previous_report.as_ref().map(|report| &report.aggregate),
            previous_report
                .as_ref()
                .map(|report| report.campaign_id.as_str()),
        );
        let regression_guard = evaluate_regression_guard(
            &RegressionGuardConfig {
                max_drawdown_regression_pct: config.max_allowed_drawdown_regression_pct,
                max_calibration_regression: config.max_allowed_calibration_regression,
                max_denial_rate_change: config.max_allowed_risk_governor_instability,
                max_no_trade_rate_change: config.max_allowed_risk_governor_instability,
                ..RegressionGuardConfig::default()
            },
            &aggregate,
            previous_report.as_ref().map(|report| &report.aggregate),
            &diff_report,
        );
        let readiness_report = build_campaign_expansion_readiness_report(
            config,
            &aggregate,
            &matrix_results,
            &diff_report,
            &regression_guard,
        );
        let mut report = ResearchCampaignReport {
            campaign_id: config.campaign_id.clone(),
            description: config.description.clone(),
            matrix_results,
            aggregate,
            diff_report,
            regression_guard,
            readiness_report,
            reason_codes,
            errors,
        };
        let output_dir = Path::new(&config.output_root).join(&config.campaign_id);
        if let Err(err) = report.write_to_dir(&output_dir) {
            report.reason_codes.push(ReasonCode::ReportWriteFailed);
            report.errors.push(err);
        }
        let store_config = EvidenceStoreConfig {
            root_path: config.evidence_store_path.clone(),
            campaign_id: config.campaign_id.clone(),
            allow_overwrite: config.allow_evidence_overwrite,
            created_at_ms: config.created_at_ms,
            reason_codes: vec![ReasonCode::DeterministicPath],
        };
        if let Err(err) =
            self.evidence_store
                .save_campaign_report(&store_config, config, &matrix_inputs, &report)
        {
            report
                .reason_codes
                .push(ReasonCode::EvidenceStoreWriteFailed);
            report.errors.push(err);
        }
        report
    }

    fn load_matrices(
        &self,
        config: &ResearchCampaignConfig,
    ) -> (
        Vec<LoadedMatrixConfig>,
        Vec<CampaignMatrixResult>,
        Vec<String>,
    ) {
        let mut loaded = Vec::new();
        let mut failures = Vec::new();
        let mut matrix_inputs = Vec::new();
        for path in &config.matrix_config_paths {
            match fs::read_to_string(path) {
                Ok(text) => match toml::from_str::<ExperimentMatrixConfig>(&text) {
                    Ok(matrix) => {
                        matrix_inputs.push(text);
                        loaded.push(LoadedMatrixConfig {
                            source: path.clone(),
                            config: matrix,
                        });
                    }
                    Err(err) => failures.push(CampaignMatrixResult {
                        matrix_id: path.clone(),
                        source: path.clone(),
                        status: CampaignMatrixStatus::Failed,
                        report: None,
                        reason_codes: vec![ReasonCode::CampaignMatrixLoadFailed],
                        error: Some(err.to_string()),
                    }),
                },
                Err(err) => failures.push(CampaignMatrixResult {
                    matrix_id: path.clone(),
                    source: path.clone(),
                    status: CampaignMatrixStatus::Failed,
                    report: None,
                    reason_codes: vec![ReasonCode::CampaignMatrixLoadFailed],
                    error: Some(err.to_string()),
                }),
            }
        }
        for matrix in &config.embedded_matrices {
            matrix_inputs.push(matrix.to_toml_string().unwrap_or_default());
            loaded.push(LoadedMatrixConfig {
                source: format!("embedded:{}", matrix.matrix_id),
                config: matrix.clone(),
            });
        }
        loaded.sort_by(|left, right| left.config.matrix_id.cmp(&right.config.matrix_id));
        matrix_inputs.sort();
        (loaded, failures, matrix_inputs)
    }

    fn load_previous_report(
        &self,
        config: &ResearchCampaignConfig,
    ) -> Option<ResearchCampaignReport> {
        if let Some(path) = &config.compare_against_report_path {
            return ResearchCampaignReport::from_json_path(Path::new(path)).ok();
        }
        let compare_id = config.compare_against_campaign_id.as_deref()?;
        let snapshots = self
            .evidence_store
            .list_snapshots(&EvidenceStoreConfig {
                root_path: config.evidence_store_path.clone(),
                campaign_id: compare_id.to_string(),
                allow_overwrite: true,
                created_at_ms: None,
                reason_codes: vec![ReasonCode::DeterministicPath],
            })
            .ok()?;
        let latest = snapshots.last()?;
        ResearchCampaignReport::from_json_path(Path::new(&latest.report_path)).ok()
    }
}

pub fn build_campaign_aggregate(
    campaign_id: &str,
    matrix_results: &[CampaignMatrixResult],
) -> CampaignAggregate {
    let reports = matrix_results
        .iter()
        .filter_map(|result| result.report.as_ref())
        .collect::<Vec<_>>();
    let run_summaries = reports
        .iter()
        .flat_map(|report| report.run_summaries.iter())
        .filter(|summary| summary.status != ExperimentRunStatus::Skipped)
        .collect::<Vec<_>>();
    let mut dataset_scores = BTreeMap::<String, (f64, DataQualitySeverity)>::new();
    for summary in &run_summaries {
        dataset_scores
            .entry(summary.run_key.dataset_id.clone())
            .and_modify(|(score, severity)| {
                *score = score.min(summary.data_quality_score);
                if severity_rank(summary.data_quality_severity) > severity_rank(*severity) {
                    *severity = summary.data_quality_severity;
                }
            })
            .or_insert((summary.data_quality_score, summary.data_quality_severity));
    }
    let quality_scores = dataset_scores
        .values()
        .map(|(score, _)| *score)
        .collect::<Vec<_>>();
    let net_returns = run_summaries
        .iter()
        .map(|summary| summary.net_return_pct)
        .collect::<Vec<_>>();
    let drawdowns = run_summaries
        .iter()
        .map(|summary| summary.max_drawdown_pct)
        .collect::<Vec<_>>();
    let profit_factors = run_summaries
        .iter()
        .filter_map(|summary| summary.profit_factor)
        .collect::<Vec<_>>();
    let calibration = run_summaries
        .iter()
        .filter_map(|summary| summary.calibration_brier)
        .collect::<Vec<_>>();
    let regime_counts = reports
        .iter()
        .flat_map(|report| report.regime_summary.counts_by_regime.keys().cloned())
        .collect::<BTreeSet<_>>();
    let regime_decision_total = reports
        .iter()
        .flat_map(|report| report.regime_summary.decisions_by_regime.values())
        .sum::<usize>();
    let unknown_regime_decisions = reports
        .iter()
        .map(|report| {
            report
                .regime_summary
                .decisions_by_regime
                .get("Unknown")
                .copied()
                .unwrap_or(0)
        })
        .sum::<usize>();
    let panic_regime_decisions = reports
        .iter()
        .map(|report| {
            report
                .regime_summary
                .decisions_by_regime
                .get("Panic")
                .copied()
                .unwrap_or(0)
        })
        .sum::<usize>();
    CampaignAggregate {
        campaign_id: campaign_id.to_string(),
        matrix_count: matrix_results.len(),
        total_runs: reports
            .iter()
            .map(|report| report.aggregate_benchmark.total_runs)
            .sum(),
        passed_runs: run_summaries
            .iter()
            .filter(|summary| summary.status == ExperimentRunStatus::Passed)
            .count(),
        failed_runs: run_summaries
            .iter()
            .filter(|summary| summary.status == ExperimentRunStatus::Failed)
            .count()
            + matrix_results
                .iter()
                .filter(|result| {
                    result.status == CampaignMatrixStatus::Failed && result.report.is_none()
                })
                .count(),
        skipped_runs: reports
            .iter()
            .map(|report| report.aggregate_benchmark.skipped_runs)
            .sum(),
        usable_dataset_count: dataset_scores
            .values()
            .filter(|(_, severity)| {
                !matches!(
                    severity,
                    DataQualitySeverity::Bad | DataQualitySeverity::Unusable
                )
            })
            .count(),
        total_dataset_count: dataset_scores.len(),
        total_outcome_records: run_summaries
            .iter()
            .map(|summary| summary.total_decisions)
            .sum(),
        total_executed_trades: run_summaries
            .iter()
            .map(|summary| summary.executed_trades)
            .sum(),
        total_no_trades: run_summaries.iter().map(|summary| summary.no_trades).sum(),
        total_denials: run_summaries
            .iter()
            .map(|summary| summary.denied_trades)
            .sum(),
        average_data_quality_score: average(&quality_scores),
        worst_data_quality_score: quality_scores
            .iter()
            .copied()
            .min_by(|left, right| left.total_cmp(right))
            .unwrap_or(0.0),
        average_net_return_pct: average(&net_returns),
        median_net_return_pct: median(&net_returns),
        worst_net_return_pct: net_returns
            .iter()
            .copied()
            .min_by(|left, right| left.total_cmp(right))
            .unwrap_or(0.0),
        average_max_drawdown_pct: average(&drawdowns),
        worst_max_drawdown_pct: drawdowns
            .iter()
            .copied()
            .max_by(|left, right| left.total_cmp(right))
            .unwrap_or(0.0),
        average_profit_factor: if profit_factors.is_empty() {
            None
        } else {
            Some(average(&profit_factors))
        },
        average_calibration_brier: if calibration.is_empty() {
            None
        } else {
            Some(average(&calibration))
        },
        regime_coverage_count: regime_counts.len(),
        unknown_regime_rate: safe_ratio(
            unknown_regime_decisions as f64,
            regime_decision_total as f64,
        ),
        panic_regime_rate: safe_ratio(panic_regime_decisions as f64, regime_decision_total as f64),
        risk_defensive_value_total: reports
            .iter()
            .map(|report| report.risk_governor_summary.defensive_value_total)
            .sum(),
        risk_opportunity_cost_total: reports
            .iter()
            .map(|report| report.risk_governor_summary.opportunity_cost_total)
            .sum(),
        persona_redundancy_warning_count: reports
            .iter()
            .filter(|report| report.persona_readiness_summary.redundancy_warning)
            .count(),
        external_model_validated_count: reports
            .iter()
            .filter(|report| {
                report.model_comparison_summary.compared_runs > 0
                    && report.model_comparison_summary.external_failed_schema_count == 0
            })
            .count(),
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn campaign_matrix_status(report: &BatchExperimentReport) -> CampaignMatrixStatus {
    if report
        .run_summaries
        .iter()
        .any(|summary| summary.status == ExperimentRunStatus::Failed)
    {
        CampaignMatrixStatus::Failed
    } else if report
        .run_summaries
        .iter()
        .any(|summary| summary.status == ExperimentRunStatus::Warning)
    {
        CampaignMatrixStatus::Warning
    } else {
        CampaignMatrixStatus::Passed
    }
}

fn minimal_report(
    config: &ResearchCampaignConfig,
    matrix_results: Vec<CampaignMatrixResult>,
    aggregate: CampaignAggregate,
    diff_report: CampaignDiffReport,
    reason_codes: Vec<ReasonCode>,
    errors: Vec<String>,
) -> ResearchCampaignReport {
    ResearchCampaignReport {
        campaign_id: config.campaign_id.clone(),
        description: config.description.clone(),
        matrix_results,
        aggregate,
        diff_report,
        regression_guard: RegressionGuardResult {
            passed: false,
            regressions: Vec::new(),
            warnings: vec!["campaign did not reach comparable evidence state".to_string()],
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        readiness_report: CampaignExpansionReadinessReport {
            decision: ExpansionReadinessDecision::NeedMoreExperiments,
            confidence: 0.90,
            evidence: CampaignExpansionReadinessEvidence {
                matrix_count: 0,
                total_dataset_count: 0,
                usable_dataset_count: 0,
                total_outcome_records: 0,
                passed_runs: 0,
                regime_coverage_count: 0,
                average_data_quality_score: 0.0,
                average_calibration_brier: None,
                worst_max_drawdown_pct: 0.0,
                risk_defensive_value_total: 0.0,
                denial_rate: 0.0,
                no_trade_rate: 0.0,
                persona_redundancy_warning_rate: 1.0,
                risk_governor_not_blocking_everything: false,
                risk_governor_not_allowing_everything: false,
                stable_feature_schema: true,
                leakage_guard_passed: true,
                no_runtime_llm: true,
                no_real_broker: true,
                no_live_api: true,
            },
            blockers: errors.clone(),
            warnings: vec!["campaign runner returned a minimal conservative report".to_string()],
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        reason_codes,
        errors,
    }
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
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.total_cmp(right));
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

fn safe_ratio(numerator: f64, denominator: f64) -> f64 {
    if denominator <= 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

fn severity_rank(severity: DataQualitySeverity) -> u8 {
    match severity {
        DataQualitySeverity::Good => 0,
        DataQualitySeverity::Warning => 1,
        DataQualitySeverity::Bad => 2,
        DataQualitySeverity::Unusable => 3,
    }
}

fn default_true() -> bool {
    true
}
