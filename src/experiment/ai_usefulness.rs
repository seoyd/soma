use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::model_gates::{ModelUsefulnessGate, ModelUsefulnessGateResult};
use super::official_coverage::OfficialDatasetCoverageReport;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiSignalStatus {
    PipelineOnly,
    BaselineEvaluated,
    ExternalModelEvaluated,
    UsefulCandidate,
    InsufficientOfficialData,
    InsufficientOutcomes,
    PoorCalibration,
    PoorRiskBehavior,
    WorseThanBaseline,
    RejectedByRisk,
    MissingOfficialData,
    MissingAuth,
    Blocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiSignalRecommendation {
    CollectMoreOfficialData,
    ImproveDataFirst,
    ImproveSignalModelFirst,
    ImproveRiskGovernorFirst,
    TuneCalibrationFirst,
    KeepBaselineOnly,
    ExternalModelCandidate,
    NeedMoreExperiments,
    MissingAuth,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub dataset_count: usize,
    pub total_trades: usize,
    pub avg_net_return_pct: f64,
    pub avg_profit_factor: f64,
    pub avg_max_drawdown_pct: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CalibrationSummary {
    pub total_count: usize,
    pub avg_brier_score: f64,
    pub avg_expected_calibration_error: f64,
    pub acceptable: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RiskGovernorSummary {
    pub total_signals: usize,
    pub denied_by_risk: usize,
    pub denial_rate: f64,
    pub approval_rate: f64,
    pub emergency_stop_count: usize,
    pub cooldown_count: usize,
    pub defensive_value: f64,
    pub opportunity_cost: f64,
    pub stable: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelComparisonSummary {
    pub compared_datasets: usize,
    pub external_better_count: usize,
    pub avg_delta_net_return_pct: f64,
    pub avg_delta_max_drawdown_pct: f64,
    pub avg_delta_profit_factor: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StorageBudgetSummary {
    pub collection_bytes: usize,
    pub dataset_export_bytes: usize,
    pub prediction_bytes: usize,
    pub report_bytes: usize,
    pub budget_exceeded: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiSignalUsefulnessReport {
    pub status: AiSignalStatus,
    pub official_dataset_count: usize,
    pub crypto_dataset_count: usize,
    pub korean_equity_dataset_count: usize,
    pub us_equity_dataset_count: usize,
    pub total_outcome_records: usize,
    pub baseline_summary: PerformanceSummary,
    #[serde(default)]
    pub external_summary: Option<PerformanceSummary>,
    pub calibration_summary: CalibrationSummary,
    pub risk_governor_summary: RiskGovernorSummary,
    #[serde(default)]
    pub model_comparison_summary: Option<ModelComparisonSummary>,
    pub storage_budget_summary: StorageBudgetSummary,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub recommendation: AiSignalRecommendation,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AiSignalDecisionInputs {
    pub official_dataset_count: usize,
    pub total_outcome_records: usize,
    pub baseline_summary: PerformanceSummary,
    pub external_summary: Option<PerformanceSummary>,
    pub calibration_summary: CalibrationSummary,
    pub risk_governor_summary: RiskGovernorSummary,
    pub model_comparison_summary: Option<ModelComparisonSummary>,
    pub storage_budget_summary: StorageBudgetSummary,
    pub has_external_evaluation: bool,
    pub comparison_external_better: bool,
    pub missing_auth: bool,
    pub non_official_ready_entries: usize,
    pub allow_upbit_only: bool,
    pub allow_equity_missing_auth: bool,
    pub min_official_ready_datasets: usize,
}

impl AiSignalUsefulnessReport {
    pub fn decide(
        coverage: &OfficialDatasetCoverageReport,
        gate_result: &ModelUsefulnessGateResult,
        inputs: AiSignalDecisionInputs,
    ) -> Self {
        let mut blockers = Vec::new();
        let mut warnings = gate_result.warnings.clone();
        let mut reason_codes = Vec::new();
        let only_crypto = coverage.crypto_ready_entries > 0
            && coverage.korean_equity_ready_entries == 0
            && coverage.us_equity_ready_entries == 0;

        if inputs.official_dataset_count < inputs.min_official_ready_datasets {
            blockers.push(format!(
                "official ready datasets {} < required {}",
                inputs.official_dataset_count, inputs.min_official_ready_datasets
            ));
        }
        if only_crypto {
            warnings.push("official evidence is crypto-only".to_string());
        }
        if coverage.korean_equity_ready_entries == 0
            && coverage
                .missing_auth_providers
                .iter()
                .any(|value| value == "krx")
        {
            warnings.push(
                "KRX readiness claim blocked by missing auth or skipped coverage".to_string(),
            );
        }
        if coverage.us_equity_ready_entries == 0
            && coverage
                .missing_auth_providers
                .iter()
                .any(|value| value == "alphavantage" || value == "alpaca")
        {
            warnings.push(
                "US equity readiness claim blocked by missing auth or skipped coverage".to_string(),
            );
        }

        let status = if inputs.official_dataset_count == 0 {
            if inputs.non_official_ready_entries > 0 {
                reason_codes.push(ReasonCode::AiSignalPipelineOnly);
                AiSignalStatus::PipelineOnly
            } else if inputs.missing_auth {
                reason_codes.push(ReasonCode::AiSignalMissingAuth);
                AiSignalStatus::MissingAuth
            } else {
                reason_codes.push(ReasonCode::AiSignalMissingOfficialData);
                AiSignalStatus::MissingOfficialData
            }
        } else if inputs.official_dataset_count < inputs.min_official_ready_datasets
            && !inputs.allow_upbit_only
        {
            reason_codes.push(ReasonCode::AiSignalInsufficientOfficialData);
            AiSignalStatus::InsufficientOfficialData
        } else if gate_result
            .failed_gates
            .contains(&ModelUsefulnessGate::EnoughOutcomes)
        {
            reason_codes.push(ReasonCode::AiSignalInsufficientOutcomes);
            AiSignalStatus::InsufficientOutcomes
        } else if gate_result
            .failed_gates
            .contains(&ModelUsefulnessGate::CalibrationAcceptable)
        {
            reason_codes.push(ReasonCode::AiSignalPoorCalibration);
            AiSignalStatus::PoorCalibration
        } else if gate_result
            .failed_gates
            .contains(&ModelUsefulnessGate::RiskGovernorStable)
        {
            reason_codes.push(ReasonCode::AiSignalPoorRiskBehavior);
            AiSignalStatus::PoorRiskBehavior
        } else if inputs.risk_governor_summary.denial_rate > 0.98 {
            reason_codes.push(ReasonCode::AiSignalPoorRiskBehavior);
            AiSignalStatus::RejectedByRisk
        } else if gate_result
            .failed_gates
            .contains(&ModelUsefulnessGate::NetReturnNotWorse)
            || (inputs.has_external_evaluation
                && inputs
                    .model_comparison_summary
                    .as_ref()
                    .is_some_and(|summary| summary.avg_delta_net_return_pct < 0.0))
        {
            reason_codes.push(ReasonCode::AiSignalWorseThanBaseline);
            AiSignalStatus::WorseThanBaseline
        } else if inputs.has_external_evaluation
            && gate_result.passed
            && inputs.comparison_external_better
        {
            reason_codes.push(ReasonCode::AiSignalUsefulCandidate);
            AiSignalStatus::UsefulCandidate
        } else if inputs.has_external_evaluation {
            reason_codes.push(ReasonCode::AiSignalExternalEvaluated);
            AiSignalStatus::ExternalModelEvaluated
        } else if inputs.baseline_summary.dataset_count != 0 {
            reason_codes.push(ReasonCode::AiSignalBaselineEvaluated);
            AiSignalStatus::BaselineEvaluated
        } else {
            reason_codes.push(ReasonCode::AiSignalPipelineOnly);
            AiSignalStatus::PipelineOnly
        };

        let recommendation = match status {
            AiSignalStatus::MissingAuth => AiSignalRecommendation::MissingAuth,
            AiSignalStatus::MissingOfficialData | AiSignalStatus::InsufficientOfficialData => {
                AiSignalRecommendation::CollectMoreOfficialData
            }
            AiSignalStatus::PipelineOnly | AiSignalStatus::InsufficientOutcomes => {
                AiSignalRecommendation::NeedMoreExperiments
            }
            AiSignalStatus::PoorCalibration => AiSignalRecommendation::TuneCalibrationFirst,
            AiSignalStatus::PoorRiskBehavior | AiSignalStatus::RejectedByRisk => {
                AiSignalRecommendation::ImproveRiskGovernorFirst
            }
            AiSignalStatus::WorseThanBaseline | AiSignalStatus::BaselineEvaluated => {
                AiSignalRecommendation::KeepBaselineOnly
            }
            AiSignalStatus::UsefulCandidate => AiSignalRecommendation::ExternalModelCandidate,
            AiSignalStatus::ExternalModelEvaluated => {
                AiSignalRecommendation::ImproveSignalModelFirst
            }
            AiSignalStatus::Blocked => AiSignalRecommendation::ImproveDataFirst,
        };

        if inputs.missing_auth && !inputs.allow_equity_missing_auth {
            blockers.push("equity provider auth is missing for part of the benchmark".to_string());
        }

        Self {
            status,
            official_dataset_count: inputs.official_dataset_count,
            crypto_dataset_count: coverage.crypto_ready_entries,
            korean_equity_dataset_count: coverage.korean_equity_ready_entries,
            us_equity_dataset_count: coverage.us_equity_ready_entries,
            total_outcome_records: inputs.total_outcome_records,
            baseline_summary: inputs.baseline_summary,
            external_summary: inputs.external_summary,
            calibration_summary: inputs.calibration_summary,
            risk_governor_summary: inputs.risk_governor_summary,
            model_comparison_summary: inputs.model_comparison_summary,
            storage_budget_summary: inputs.storage_budget_summary,
            blockers,
            warnings,
            recommendation,
            reason_codes,
        }
    }

    pub fn to_markdown(&self) -> String {
        [
            "| field | value |".to_string(),
            "| --- | --- |".to_string(),
            format!("| status | {:?} |", self.status),
            format!(
                "| official_dataset_count | {} |",
                self.official_dataset_count
            ),
            format!("| total_outcome_records | {} |", self.total_outcome_records),
            format!("| recommendation | {:?} |", self.recommendation),
            format!(
                "| calibration | brier={:.6}, ece={:.6}, acceptable={} |",
                self.calibration_summary.avg_brier_score,
                self.calibration_summary.avg_expected_calibration_error,
                self.calibration_summary.acceptable
            ),
            format!(
                "| risk | denial_rate={:.6}, approval_rate={:.6}, stable={} |",
                self.risk_governor_summary.denial_rate,
                self.risk_governor_summary.approval_rate,
                self.risk_governor_summary.stable
            ),
        ]
        .join("\n")
    }
}
