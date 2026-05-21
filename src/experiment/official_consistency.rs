use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::ai_benchmark::OfficialAiBenchmarkReport;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialConsistencyConfig {
    pub consistency_id: String,
    pub official_benchmark_report_paths: Vec<String>,
    #[serde(default)]
    pub campaign_report_paths: Vec<String>,
    #[serde(default = "default_true")]
    pub require_real_official_data: bool,
    pub min_crypto_datasets: usize,
    pub min_korean_equity_datasets: usize,
    pub min_us_equity_datasets: usize,
    pub min_total_outcomes: usize,
    pub min_per_venue_outcomes: usize,
    pub max_allowed_metric_variance: f64,
    pub max_allowed_drawdown_variance: f64,
    pub max_allowed_calibration_variance: f64,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl OfficialConsistencyConfig {
    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        if self
            .official_benchmark_report_paths
            .iter()
            .chain(self.campaign_report_paths.iter())
            .any(|path| path.contains("://"))
        {
            vec![ReasonCode::LocalPathRejected]
        } else {
            Vec::new()
        }
    }

    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn load_benchmark_reports(&self) -> Result<Vec<OfficialAiBenchmarkReport>, String> {
        self.official_benchmark_report_paths
            .iter()
            .map(|path| OfficialAiBenchmarkReport::from_json_path(Path::new(path)))
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfficialConsistencyStatus {
    ConsistentEnough,
    CryptoOnly,
    MissingEquityData,
    MissingAuth,
    InsufficientOutcomes,
    InconsistentMetrics,
    PoorCalibration,
    PoorRiskBehavior,
    NeedMoreExperiments,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialConsistencyReport {
    pub consistency_id: String,
    pub crypto_summary: String,
    pub korean_equity_summary: String,
    pub us_equity_summary: String,
    pub skipped_provider_summary: String,
    pub auth_gap_summary: String,
    pub metric_stability_summary: String,
    pub calibration_stability_summary: String,
    pub risk_stability_summary: String,
    pub storage_budget_summary: String,
    pub consistency_status: OfficialConsistencyStatus,
    pub reason_codes: Vec<ReasonCode>,
}

impl OfficialConsistencyReport {
    pub fn build(
        config: &OfficialConsistencyConfig,
        reports: &[OfficialAiBenchmarkReport],
    ) -> Self {
        if reports.is_empty() {
            return Self {
                consistency_id: config.consistency_id.clone(),
                crypto_summary: "crypto_ready=0".to_string(),
                korean_equity_summary: "korean_equity_ready=0".to_string(),
                us_equity_summary: "us_equity_ready=0".to_string(),
                skipped_provider_summary: "reports=0".to_string(),
                auth_gap_summary: "missing_auth_providers=".to_string(),
                metric_stability_summary: "metric_variance=0.000000".to_string(),
                calibration_stability_summary: "calibration_variance=0.000000".to_string(),
                risk_stability_summary: "risk_variance=0.000000".to_string(),
                storage_budget_summary: "budget_exceeded=false".to_string(),
                consistency_status: OfficialConsistencyStatus::NeedMoreExperiments,
                reason_codes: vec![ReasonCode::OfficialConsistencyBuilt],
            };
        }
        let crypto_datasets: usize = reports
            .iter()
            .map(|report| report.usefulness_report.crypto_dataset_count)
            .sum();
        let korean_equity_datasets: usize = reports
            .iter()
            .map(|report| report.usefulness_report.korean_equity_dataset_count)
            .sum();
        let us_equity_datasets: usize = reports
            .iter()
            .map(|report| report.usefulness_report.us_equity_dataset_count)
            .sum();
        let total_outcomes: usize = reports
            .iter()
            .map(|report| report.usefulness_report.total_outcome_records)
            .sum();
        let crypto_outcomes: usize = reports
            .iter()
            .filter(|report| report.usefulness_report.crypto_dataset_count > 0)
            .map(|report| report.usefulness_report.total_outcome_records)
            .sum();
        let korean_outcomes: usize = reports
            .iter()
            .filter(|report| report.usefulness_report.korean_equity_dataset_count > 0)
            .map(|report| report.usefulness_report.total_outcome_records)
            .sum();
        let us_outcomes: usize = reports
            .iter()
            .filter(|report| report.usefulness_report.us_equity_dataset_count > 0)
            .map(|report| report.usefulness_report.total_outcome_records)
            .sum();
        let missing_auth = reports
            .iter()
            .flat_map(|report| report.coverage_report.missing_auth_providers.clone())
            .collect::<Vec<_>>();
        let baseline_metrics = reports
            .iter()
            .map(|report| report.usefulness_report.baseline_summary.avg_net_return_pct)
            .collect::<Vec<_>>();
        let baseline_drawdown = reports
            .iter()
            .map(|report| {
                report
                    .usefulness_report
                    .baseline_summary
                    .avg_max_drawdown_pct
            })
            .collect::<Vec<_>>();
        let calibrations = reports
            .iter()
            .map(|report| report.usefulness_report.calibration_summary.avg_brier_score)
            .collect::<Vec<_>>();
        let risk_denials = reports
            .iter()
            .map(|report| report.usefulness_report.risk_governor_summary.denial_rate)
            .collect::<Vec<_>>();
        let metric_variance = spread(&baseline_metrics);
        let drawdown_variance = spread(&baseline_drawdown);
        let calibration_variance = spread(&calibrations);
        let risk_variance = spread(&risk_denials);
        let any_budget_exceeded = reports
            .iter()
            .any(|report| report.storage_audit.budget_exceeded);
        let status = if !missing_auth.is_empty() {
            OfficialConsistencyStatus::MissingAuth
        } else if korean_equity_datasets == 0 && us_equity_datasets == 0 && crypto_datasets > 0 {
            OfficialConsistencyStatus::CryptoOnly
        } else if korean_equity_datasets < config.min_korean_equity_datasets
            || us_equity_datasets < config.min_us_equity_datasets
        {
            OfficialConsistencyStatus::MissingEquityData
        } else if total_outcomes < config.min_total_outcomes
            || (crypto_datasets > 0 && crypto_outcomes < config.min_per_venue_outcomes)
            || (korean_equity_datasets > 0 && korean_outcomes < config.min_per_venue_outcomes)
            || (us_equity_datasets > 0 && us_outcomes < config.min_per_venue_outcomes)
        {
            OfficialConsistencyStatus::InsufficientOutcomes
        } else if calibration_variance > config.max_allowed_calibration_variance {
            OfficialConsistencyStatus::PoorCalibration
        } else if risk_variance > config.max_allowed_metric_variance
            || reports.iter().any(|report| {
                matches!(
                    report.usefulness_report.status,
                    crate::experiment::AiSignalStatus::PoorRiskBehavior
                        | crate::experiment::AiSignalStatus::RejectedByRisk
                )
            })
        {
            OfficialConsistencyStatus::PoorRiskBehavior
        } else if metric_variance > config.max_allowed_metric_variance
            || drawdown_variance > config.max_allowed_drawdown_variance
        {
            OfficialConsistencyStatus::InconsistentMetrics
        } else {
            OfficialConsistencyStatus::ConsistentEnough
        };
        let mut reason_codes = vec![ReasonCode::OfficialConsistencyBuilt];
        match status {
            OfficialConsistencyStatus::CryptoOnly => {
                reason_codes.push(ReasonCode::OfficialConsistencyCryptoOnly)
            }
            OfficialConsistencyStatus::MissingEquityData => {
                reason_codes.push(ReasonCode::OfficialConsistencyMissingEquityData)
            }
            OfficialConsistencyStatus::MissingAuth => {
                reason_codes.push(ReasonCode::OfficialConsistencyMissingAuth)
            }
            OfficialConsistencyStatus::InsufficientOutcomes => {
                reason_codes.push(ReasonCode::OfficialConsistencyInsufficientOutcomes)
            }
            OfficialConsistencyStatus::InconsistentMetrics => {
                reason_codes.push(ReasonCode::OfficialConsistencyInconsistentMetrics)
            }
            OfficialConsistencyStatus::PoorCalibration => {
                reason_codes.push(ReasonCode::OfficialConsistencyPoorCalibration)
            }
            OfficialConsistencyStatus::PoorRiskBehavior => {
                reason_codes.push(ReasonCode::OfficialConsistencyPoorRiskBehavior)
            }
            OfficialConsistencyStatus::ConsistentEnough
            | OfficialConsistencyStatus::NeedMoreExperiments => {}
        }

        Self {
            consistency_id: config.consistency_id.clone(),
            crypto_summary: format!(
                "datasets={crypto_datasets};outcomes={crypto_outcomes};min_required={}",
                config.min_crypto_datasets
            ),
            korean_equity_summary: format!(
                "datasets={korean_equity_datasets};outcomes={korean_outcomes};min_required={}",
                config.min_korean_equity_datasets
            ),
            us_equity_summary: format!(
                "datasets={us_equity_datasets};outcomes={us_outcomes};min_required={}",
                config.min_us_equity_datasets
            ),
            skipped_provider_summary: format!(
                "reports={};single_symbol_per_venue_only=true",
                reports.len()
            ),
            auth_gap_summary: format!("missing_auth_providers={}", missing_auth.join("|")),
            metric_stability_summary: format!(
                "metric_variance={metric_variance:.6};drawdown_variance={drawdown_variance:.6}"
            ),
            calibration_stability_summary: format!(
                "calibration_variance={calibration_variance:.6}"
            ),
            risk_stability_summary: format!("risk_variance={risk_variance:.6}"),
            storage_budget_summary: format!(
                "budget_exceeded={any_budget_exceeded};collection_bytes={}",
                reports
                    .iter()
                    .map(|report| report.storage_audit.collection_bytes)
                    .sum::<usize>()
            ),
            consistency_status: status,
            reason_codes,
        }
    }

    pub fn to_text(&self) -> String {
        [
            format!("consistency_id={}", self.consistency_id),
            format!("status={:?}", self.consistency_status),
            format!("crypto_summary={}", self.crypto_summary),
            format!("korean_equity_summary={}", self.korean_equity_summary),
            format!("us_equity_summary={}", self.us_equity_summary),
            format!("skipped_provider_summary={}", self.skipped_provider_summary),
            format!("auth_gap_summary={}", self.auth_gap_summary),
            format!("metric_stability_summary={}", self.metric_stability_summary),
            format!(
                "calibration_stability_summary={}",
                self.calibration_stability_summary
            ),
            format!("risk_stability_summary={}", self.risk_stability_summary),
            format!("storage_budget_summary={}", self.storage_budget_summary),
        ]
        .join("\n")
    }
}

fn spread(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        max - min
    }
}

fn default_true() -> bool {
    true
}
