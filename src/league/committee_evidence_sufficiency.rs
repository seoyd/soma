use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::committee_counterfactual_audit::CommitteeCounterfactualAuditReport;
use super::committee_outcome_coverage::CommitteeOutcomeCoverageReport;
use super::committee_performance_matrix::CommitteePerformanceEvidenceMatrix;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeEvidenceSufficiencyGateConfig {
    #[serde(default = "default_min_total_rows")]
    pub min_total_rows: usize,
    #[serde(default = "default_min_official_rows")]
    pub min_official_rows: usize,
    #[serde(default = "default_min_outcome_linked_rows")]
    pub min_outcome_linked_rows: usize,
    #[serde(default = "default_min_baseline_references")]
    pub min_baseline_references: usize,
    #[serde(default = "default_min_counterfactuals")]
    pub min_no_trade_counterfactuals: usize,
    #[serde(default = "default_min_counterfactuals")]
    pub min_risk_denied_counterfactuals: usize,
    #[serde(default = "default_min_row_level_ratio")]
    pub min_row_level_ratio: f64,
    #[serde(default = "default_max_summary_derived_ratio")]
    pub max_summary_derived_ratio: f64,
    #[serde(default = "default_max_research_only_ratio")]
    pub max_research_only_ratio: f64,
    #[serde(default = "default_max_fixture_ratio")]
    pub max_fixture_ratio: f64,
    #[serde(default = "default_true")]
    pub require_no_lookahead_safe: bool,
    #[serde(default)]
    pub require_source_diversity: bool,
    #[serde(default)]
    pub require_non_crypto_official: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeEvidenceSufficiencyStatus {
    SufficientForCommitteeBenchmark,
    SufficientForCryptoOnlyBenchmark,
    SufficientForDiagnosticsOnly,
    InsufficientOfficialRows,
    InsufficientOutcomeLinks,
    InsufficientCounterfactuals,
    InsufficientBaselineReferences,
    TooMuchSummaryDerived,
    TooMuchResearchOnly,
    TooMuchFixture,
    NoLookaheadViolation,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeEvidenceSufficiencyGateResult {
    pub passed: bool,
    pub failed_gates: Vec<String>,
    pub warnings: Vec<String>,
    pub sufficiency_status: CommitteeEvidenceSufficiencyStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for CommitteeEvidenceSufficiencyGateConfig {
    fn default() -> Self {
        Self {
            min_total_rows: default_min_total_rows(),
            min_official_rows: default_min_official_rows(),
            min_outcome_linked_rows: default_min_outcome_linked_rows(),
            min_baseline_references: default_min_baseline_references(),
            min_no_trade_counterfactuals: default_min_counterfactuals(),
            min_risk_denied_counterfactuals: default_min_counterfactuals(),
            min_row_level_ratio: default_min_row_level_ratio(),
            max_summary_derived_ratio: default_max_summary_derived_ratio(),
            max_research_only_ratio: default_max_research_only_ratio(),
            max_fixture_ratio: default_max_fixture_ratio(),
            require_no_lookahead_safe: true,
            require_source_diversity: false,
            require_non_crypto_official: false,
            reason_codes: vec![ReasonCode::CommitteeEvidenceSufficiencyGateBuilt],
        }
    }
}

pub fn evaluate_committee_evidence_sufficiency(
    config: &CommitteeEvidenceSufficiencyGateConfig,
    coverage_report: &CommitteeOutcomeCoverageReport,
    counterfactual_audit_report: Option<&CommitteeCounterfactualAuditReport>,
    performance_matrix: &CommitteePerformanceEvidenceMatrix,
) -> CommitteeEvidenceSufficiencyGateResult {
    let mut failed_gates = Vec::new();
    let mut warnings = Vec::new();
    let no_trade_counterfactuals = counterfactual_audit_report
        .map(|report| report.no_trade_count)
        .unwrap_or(coverage_report.no_trade_counterfactuals);
    let risk_denied_counterfactuals = counterfactual_audit_report
        .map(|report| report.risk_denied_count)
        .unwrap_or(coverage_report.risk_denied_counterfactuals);

    if coverage_report.total_rows < config.min_total_rows {
        failed_gates.push("min_total_rows".to_string());
    }
    if coverage_report.official_rows < config.min_official_rows {
        failed_gates.push("min_official_rows".to_string());
    }
    if coverage_report.outcome_linked_rows < config.min_outcome_linked_rows {
        failed_gates.push("min_outcome_linked_rows".to_string());
    }
    if coverage_report.baseline_linked_rows < config.min_baseline_references {
        failed_gates.push("min_baseline_references".to_string());
    }
    if no_trade_counterfactuals < config.min_no_trade_counterfactuals {
        failed_gates.push("min_no_trade_counterfactuals".to_string());
    }
    if risk_denied_counterfactuals < config.min_risk_denied_counterfactuals {
        failed_gates.push("min_risk_denied_counterfactuals".to_string());
    }
    if coverage_report.row_level_ratio() < config.min_row_level_ratio {
        warnings.push("row_level_ratio below conservative threshold".to_string());
    }
    if coverage_report.summary_derived_ratio() > config.max_summary_derived_ratio {
        failed_gates.push("max_summary_derived_ratio".to_string());
    }
    if coverage_report.research_only_ratio() > config.max_research_only_ratio {
        failed_gates.push("max_research_only_ratio".to_string());
    }
    if coverage_report.fixture_ratio() > config.max_fixture_ratio {
        failed_gates.push("max_fixture_ratio".to_string());
    }
    if config.require_no_lookahead_safe && coverage_report.no_lookahead_violations > 0 {
        failed_gates.push("require_no_lookahead_safe".to_string());
    }
    if config.require_source_diversity && coverage_report.source_diversity_count() < 2 {
        failed_gates.push("require_source_diversity".to_string());
    }
    if config.require_non_crypto_official && coverage_report.official_non_crypto_rows() == 0 {
        failed_gates.push("require_non_crypto_official".to_string());
    }
    if performance_matrix.total_comparable_rows == 0 {
        warnings.push("no performance comparables available".to_string());
    }
    warnings.push(
        "sufficiency pass remains research-only and does not imply profitability or live readiness"
            .to_string(),
    );
    warnings.push(
        "six-person design review stays stricter and report-only than benchmark sufficiency"
            .to_string(),
    );

    let passed = failed_gates.is_empty();
    let sufficiency_status = if passed && coverage_report.crypto_only_ratio() >= 0.999 {
        CommitteeEvidenceSufficiencyStatus::SufficientForCryptoOnlyBenchmark
    } else if passed
        && (coverage_report.research_only_ratio() > 0.0
            || coverage_report.fixture_ratio() > 0.0
            || performance_matrix.total_comparable_rows == 0)
    {
        CommitteeEvidenceSufficiencyStatus::SufficientForDiagnosticsOnly
    } else if passed {
        CommitteeEvidenceSufficiencyStatus::SufficientForCommitteeBenchmark
    } else if failed_gates
        .iter()
        .any(|gate| gate == "require_no_lookahead_safe")
    {
        CommitteeEvidenceSufficiencyStatus::NoLookaheadViolation
    } else if failed_gates.iter().any(|gate| gate == "min_official_rows") {
        CommitteeEvidenceSufficiencyStatus::InsufficientOfficialRows
    } else if failed_gates
        .iter()
        .any(|gate| gate == "min_outcome_linked_rows")
    {
        CommitteeEvidenceSufficiencyStatus::InsufficientOutcomeLinks
    } else if failed_gates
        .iter()
        .any(|gate| gate == "min_baseline_references")
    {
        CommitteeEvidenceSufficiencyStatus::InsufficientBaselineReferences
    } else if failed_gates
        .iter()
        .any(|gate| gate.contains("counterfactual"))
    {
        CommitteeEvidenceSufficiencyStatus::InsufficientCounterfactuals
    } else if failed_gates
        .iter()
        .any(|gate| gate == "max_summary_derived_ratio")
    {
        CommitteeEvidenceSufficiencyStatus::TooMuchSummaryDerived
    } else if failed_gates
        .iter()
        .any(|gate| gate == "max_research_only_ratio")
    {
        CommitteeEvidenceSufficiencyStatus::TooMuchResearchOnly
    } else if failed_gates.iter().any(|gate| gate == "max_fixture_ratio") {
        CommitteeEvidenceSufficiencyStatus::TooMuchFixture
    } else {
        CommitteeEvidenceSufficiencyStatus::NeedMoreEvidence
    };

    CommitteeEvidenceSufficiencyGateResult {
        passed,
        failed_gates,
        warnings,
        sufficiency_status,
        reason_codes: stable_reason_codes(
            &config
                .reason_codes
                .iter()
                .cloned()
                .chain([ReasonCode::CommitteeEvidenceSufficiencyGateBuilt])
                .collect::<Vec<_>>(),
        ),
    }
}

impl CommitteeEvidenceSufficiencyGateResult {
    pub fn to_text(&self) -> String {
        [
            format!("passed={}", self.passed),
            format!("sufficiency_status={:?}", self.sufficiency_status),
            format!("failed_gates={}", self.failed_gates.join("|")),
            format!("warnings={}", self.warnings.join("|")),
        ]
        .join("\n")
    }
}

fn default_min_total_rows() -> usize {
    3
}

fn default_min_official_rows() -> usize {
    3
}

fn default_min_outcome_linked_rows() -> usize {
    3
}

fn default_min_baseline_references() -> usize {
    3
}

fn default_min_counterfactuals() -> usize {
    1
}

fn default_min_row_level_ratio() -> f64 {
    0.50
}

fn default_max_summary_derived_ratio() -> f64 {
    0.40
}

fn default_max_research_only_ratio() -> f64 {
    0.0
}

fn default_max_fixture_ratio() -> f64 {
    0.0
}

fn default_true() -> bool {
    true
}
