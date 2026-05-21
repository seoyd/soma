use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{
    CoreLatencyBudgetReport, CoreLatencyBudgetStatus, ReasonCode, stable_hash_string,
    stable_reason_codes,
};
use crate::experiment::{
    CoreBottleneckRecommendation, CoreBottleneckReport, CorePerformanceArtifactInventory,
    CorePerformanceRegressionReport, SignalQualityReport,
};
use crate::league::CommitteeValueAttributionReport;
use crate::risk::{NoTradeValueReport, RiskGovernorValueReport};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CorePerformanceFinalStatus {
    CorePerformanceHealthyForResearch,
    CoreBlockedByEvidence,
    CoreBlockedByOutcomeLinks,
    CoreBlockedByOfficialData,
    CoreBlockedByRiskBehavior,
    CoreBlockedByCalibration,
    CoreBlockedByBudget,
    #[default]
    CoreDiagnosticOnly,
    CoreNeedsMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorePerformanceScorecardConfig {
    pub scorecard_id: String,
    #[serde(default)]
    pub core_check_report_paths: Vec<String>,
    #[serde(default)]
    pub official_replication_report_paths: Vec<String>,
    #[serde(default)]
    pub committee_official_benchmark_paths: Vec<String>,
    #[serde(default)]
    pub committee_outcome_coverage_paths: Vec<String>,
    #[serde(default)]
    pub committee_reference_pack_paths: Vec<String>,
    #[serde(default)]
    pub committee_benchmark_bundle_paths: Vec<String>,
    #[serde(default)]
    pub source_aware_benchmark_paths: Vec<String>,
    #[serde(default)]
    pub yahoo_research_report_paths: Vec<String>,
    #[serde(default)]
    pub previous_scorecard_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_artifacts")]
    pub max_artifacts: usize,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub require_core_check_pass: bool,
    #[serde(default = "default_true")]
    pub require_official_for_usefulness_claim: bool,
    #[serde(default = "default_true")]
    pub allow_controlled_evidence: bool,
    #[serde(default = "default_true")]
    pub allow_crypto_only: bool,
    #[serde(default = "default_true")]
    pub allow_yfinance_research: bool,
    #[serde(default = "default_true")]
    pub allow_fixture: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CorePerformanceScorecard {
    pub scorecard_id: String,
    pub artifact_inventory: CorePerformanceArtifactInventory,
    pub signal_quality_report: SignalQualityReport,
    pub committee_value_attribution_report: CommitteeValueAttributionReport,
    pub risk_governor_value_report: RiskGovernorValueReport,
    pub no_trade_value_report: NoTradeValueReport,
    pub latency_budget_report: CoreLatencyBudgetReport,
    #[serde(default)]
    pub regression_report: Option<CorePerformanceRegressionReport>,
    pub bottleneck_report: CoreBottleneckReport,
    pub final_status: CorePerformanceFinalStatus,
    pub final_recommendation: CoreBottleneckRecommendation,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for CorePerformanceScorecardConfig {
    fn default() -> Self {
        Self {
            scorecard_id: "core-performance".to_string(),
            core_check_report_paths: Vec::new(),
            official_replication_report_paths: Vec::new(),
            committee_official_benchmark_paths: Vec::new(),
            committee_outcome_coverage_paths: Vec::new(),
            committee_reference_pack_paths: Vec::new(),
            committee_benchmark_bundle_paths: Vec::new(),
            source_aware_benchmark_paths: Vec::new(),
            yahoo_research_report_paths: Vec::new(),
            previous_scorecard_paths: Vec::new(),
            output_root: default_output_root(),
            max_artifacts: default_max_artifacts(),
            max_rows: default_max_rows(),
            max_bytes: default_max_bytes(),
            require_core_check_pass: true,
            require_official_for_usefulness_claim: true,
            allow_controlled_evidence: true,
            allow_crypto_only: true,
            allow_yfinance_research: true,
            allow_fixture: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl CorePerformanceScorecardConfig {
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
        if self
            .all_artifact_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            vec![ReasonCode::RemotePathRejected]
        } else {
            Vec::new()
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.scorecard_id.trim().is_empty() {
            return Err("core-performance scorecard_id must not be empty".to_string());
        }
        if !self.validate_local_paths().is_empty() {
            return Err("core-performance paths must be local".to_string());
        }
        if self.max_artifacts == 0 || self.max_artifacts > 128 {
            return Err("core-performance max_artifacts must be between 1 and 128".to_string());
        }
        if self.max_rows == 0 || self.max_rows > 100_000 {
            return Err("core-performance max_rows must be between 1 and 100000".to_string());
        }
        if self.max_bytes == 0 || self.max_bytes > 20_000_000 {
            return Err("core-performance max_bytes must be between 1 and 20000000".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.scorecard_id)
    }

    pub fn all_artifact_paths(&self) -> Vec<String> {
        self.core_check_report_paths
            .iter()
            .chain(self.official_replication_report_paths.iter())
            .chain(self.committee_official_benchmark_paths.iter())
            .chain(self.committee_outcome_coverage_paths.iter())
            .chain(self.committee_reference_pack_paths.iter())
            .chain(self.committee_benchmark_bundle_paths.iter())
            .chain(self.source_aware_benchmark_paths.iter())
            .chain(self.yahoo_research_report_paths.iter())
            .chain(self.previous_scorecard_paths.iter())
            .cloned()
            .collect()
    }
}

impl CorePerformanceScorecard {
    pub fn to_text(&self) -> String {
        let mut lines = self.base_lines();
        lines.push(format!("fingerprint={}", self.fingerprint()));
        lines.join("\n")
    }

    pub fn fingerprint(&self) -> String {
        stable_hash_string(&self.base_lines().join("\n"))
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    fn base_lines(&self) -> Vec<String> {
        vec![
            format!("scorecard_id={}", self.scorecard_id),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
            format!(
                "artifact_count={}",
                self.artifact_inventory.descriptors.len()
            ),
            format!(
                "official_artifact_count={}",
                self.artifact_inventory.non_crypto_official_count
            ),
            format!(
                "signal_quality_status={:?}",
                self.signal_quality_report.signal_quality_status
            ),
            format!(
                "committee_value_status={:?}",
                self.committee_value_attribution_report.attribution_status
            ),
            format!(
                "risk_governor_status={:?}",
                self.risk_governor_value_report.status
            ),
            format!("no_trade_status={:?}", self.no_trade_value_report.status),
            format!(
                "latency_budget_status={:?}",
                self.latency_budget_report.budget_status
            ),
            format!("bottleneck={:?}", self.bottleneck_report.primary_bottleneck),
        ]
    }
}

pub fn build_core_performance_scorecard(
    config: &CorePerformanceScorecardConfig,
    artifact_inventory: CorePerformanceArtifactInventory,
    signal_quality_report: SignalQualityReport,
    committee_value_attribution_report: CommitteeValueAttributionReport,
    risk_governor_value_report: RiskGovernorValueReport,
    no_trade_value_report: NoTradeValueReport,
    latency_budget_report: CoreLatencyBudgetReport,
    regression_report: Option<CorePerformanceRegressionReport>,
    bottleneck_report: CoreBottleneckReport,
    load_warnings: Vec<String>,
) -> CorePerformanceScorecard {
    let mut blockers = signal_quality_report.blockers.clone();
    blockers.extend(committee_value_attribution_report.blockers.clone());
    if let Some(regression) = &regression_report {
        blockers.extend(regression.regressions.clone());
    }

    let mut warnings = load_warnings;
    warnings.extend(signal_quality_report.warnings.clone());
    warnings.extend(committee_value_attribution_report.warnings.clone());
    warnings.extend(risk_governor_value_report.warnings.clone());
    warnings.extend(no_trade_value_report.warnings.clone());

    let final_status = if matches!(
        latency_budget_report.budget_status,
        CoreLatencyBudgetStatus::StorageBudgetExceeded
            | CoreLatencyBudgetStatus::LatencyBudgetExceeded
            | CoreLatencyBudgetStatus::TooManyArtifacts
            | CoreLatencyBudgetStatus::TooManyRows
    ) {
        CorePerformanceFinalStatus::CoreBlockedByBudget
    } else if matches!(
        bottleneck_report.primary_bottleneck,
        crate::experiment::CoreBottleneckKind::MissingOfficialAuth
            | crate::experiment::CoreBottleneckKind::MissingOfficialData
            | crate::experiment::CoreBottleneckKind::MissingOfficialCandles
    ) {
        if artifact_inventory.non_crypto_official_count == 0
            && (artifact_inventory.controlled_only_count > 0
                || artifact_inventory.research_only_count > 0
                || artifact_inventory.fixture_only_count > 0
                || artifact_inventory.crypto_only_count > 0)
        {
            CorePerformanceFinalStatus::CoreDiagnosticOnly
        } else {
            CorePerformanceFinalStatus::CoreBlockedByOfficialData
        }
    } else if bottleneck_report.primary_bottleneck
        == crate::experiment::CoreBottleneckKind::MissingOutcomeLinks
    {
        CorePerformanceFinalStatus::CoreBlockedByOutcomeLinks
    } else if matches!(
        bottleneck_report.primary_bottleneck,
        crate::experiment::CoreBottleneckKind::RiskOverBlocking
            | crate::experiment::CoreBottleneckKind::RiskUnderBlocking
    ) {
        CorePerformanceFinalStatus::CoreBlockedByRiskBehavior
    } else if bottleneck_report.primary_bottleneck
        == crate::experiment::CoreBottleneckKind::PoorCalibration
    {
        CorePerformanceFinalStatus::CoreBlockedByCalibration
    } else if artifact_inventory.non_crypto_official_count == 0
        && (artifact_inventory.controlled_only_count > 0
            || artifact_inventory.research_only_count > 0
            || artifact_inventory.fixture_only_count > 0
            || artifact_inventory.crypto_only_count > 0)
    {
        CorePerformanceFinalStatus::CoreDiagnosticOnly
    } else if !blockers.is_empty() {
        CorePerformanceFinalStatus::CoreBlockedByEvidence
    } else if artifact_inventory.non_crypto_official_count == 0 {
        CorePerformanceFinalStatus::CoreDiagnosticOnly
    } else if signal_quality_report.official_evaluated_rows == 0 {
        CorePerformanceFinalStatus::CoreNeedsMoreEvidence
    } else {
        CorePerformanceFinalStatus::CorePerformanceHealthyForResearch
    };

    let mut reason_codes = config.reason_codes.clone();
    reason_codes.push(ReasonCode::CorePerformanceScorecardBuilt);
    if final_status == CorePerformanceFinalStatus::CoreDiagnosticOnly {
        reason_codes.push(ReasonCode::CorePerformanceDiagnosticOnly);
    }
    if config.require_official_for_usefulness_claim
        && signal_quality_report.official_evaluated_rows == 0
    {
        reason_codes.push(ReasonCode::CorePerformanceUsefulnessUnproven);
    }

    CorePerformanceScorecard {
        scorecard_id: config.scorecard_id.clone(),
        artifact_inventory,
        signal_quality_report,
        committee_value_attribution_report,
        risk_governor_value_report,
        no_trade_value_report,
        latency_budget_report,
        regression_report,
        bottleneck_report: bottleneck_report.clone(),
        final_status,
        final_recommendation: bottleneck_report.recommended_next_action,
        blockers,
        warnings,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn default_output_root() -> String {
    "target/soma_core_performance".to_string()
}

fn default_max_artifacts() -> usize {
    64
}

fn default_max_rows() -> usize {
    10_000
}

fn default_max_bytes() -> usize {
    5_000_000
}

fn default_true() -> bool {
    true
}
