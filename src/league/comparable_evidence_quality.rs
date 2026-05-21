use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
    ComparableEvidenceSourceClass,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComparableEvidenceQualityStatus {
    HealthyComparableEvidence,
    NeedMoreOfficialComparableRows,
    NeedMoreOutcomeReferences,
    NeedMoreBaselineReferences,
    NeedMoreNoTradeCounterfactuals,
    NeedMoreRiskDeniedCounterfactuals,
    TooMuchSummaryDerived,
    TooMuchDiagnosticOnly,
    ControlledOnly,
    CryptoOnly,
    ResearchOnly,
    FixtureOnly,
    #[default]
    InsufficientComparableEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComparableEvidenceQualityReport {
    pub comparable_id: String,
    pub total_rows: usize,
    pub complete_rows: usize,
    pub official_complete_rows: usize,
    pub row_level_ratio: f64,
    pub summary_derived_ratio: f64,
    pub diagnostic_only_ratio: f64,
    pub outcome_reference_ratio: f64,
    pub baseline_reference_ratio: f64,
    pub no_trade_counterfactual_ratio: f64,
    pub risk_denied_counterfactual_ratio: f64,
    pub no_lookahead_safe_ratio: f64,
    pub quality_status: ComparableEvidenceQualityStatus,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_comparable_evidence_quality_report(
    config: &ComparableCommitteeEvidenceConfig,
    bundle: &ComparableCommitteeEvidenceBundle,
) -> ComparableEvidenceQualityReport {
    let total_rows = bundle.rows.len();
    let complete_rows = bundle.complete_rows;
    let official_complete_rows = bundle
        .rows
        .iter()
        .filter(|row| row.official_complete(config))
        .count();
    let row_level_ratio = ratio(bundle.row_level_count, total_rows);
    let summary_derived_ratio = ratio(bundle.summary_derived_count, total_rows);
    let diagnostic_only_ratio = ratio(
        bundle.rows.iter().filter(|row| row.diagnostic_only).count(),
        total_rows,
    );
    let outcome_reference_ratio = ratio(bundle.outcome_reference_count, total_rows);
    let baseline_reference_ratio = ratio(bundle.baseline_reference_count, total_rows);
    let no_trade_counterfactual_ratio = ratio(bundle.no_trade_counterfactual_count, total_rows);
    let risk_denied_counterfactual_ratio =
        ratio(bundle.risk_denied_counterfactual_count, total_rows);
    let no_lookahead_safe_ratio = ratio(bundle.no_lookahead_safe_count, total_rows);

    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let quality_status = if total_rows == 0 {
        blockers.push("no comparable rows were built".to_string());
        ComparableEvidenceQualityStatus::InsufficientComparableEvidence
    } else if only_source_class(bundle, ComparableEvidenceSourceClass::ControlledDiagnostic) {
        warnings.push("controlled evidence remains diagnostic-only".to_string());
        ComparableEvidenceQualityStatus::ControlledOnly
    } else if only_source_class(bundle, ComparableEvidenceSourceClass::OfficialCryptoOnly) {
        warnings.push("crypto-only evidence cannot support stock readiness".to_string());
        ComparableEvidenceQualityStatus::CryptoOnly
    } else if only_source_class(bundle, ComparableEvidenceSourceClass::YFinanceResearch) {
        warnings.push("yfinance evidence remains research-only".to_string());
        ComparableEvidenceQualityStatus::ResearchOnly
    } else if bundle.rows.iter().all(|row| {
        matches!(
            row.source_class,
            ComparableEvidenceSourceClass::FixtureArchitectureTest
                | ComparableEvidenceSourceClass::SyntheticTest
        )
    }) {
        warnings.push("fixture evidence remains architecture-test-only".to_string());
        ComparableEvidenceQualityStatus::FixtureOnly
    } else if config.require_official_for_usefulness_claim && official_complete_rows == 0 {
        blockers.push("official complete comparable rows are still missing".to_string());
        ComparableEvidenceQualityStatus::NeedMoreOfficialComparableRows
    } else if config.require_outcome_reference && outcome_reference_ratio < 1.0 {
        blockers.push("outcome references remain missing for some comparable rows".to_string());
        ComparableEvidenceQualityStatus::NeedMoreOutcomeReferences
    } else if config.require_baseline_reference && baseline_reference_ratio < 1.0 {
        blockers.push("baseline references remain missing for some comparable rows".to_string());
        ComparableEvidenceQualityStatus::NeedMoreBaselineReferences
    } else if config.require_no_trade_counterfactual && no_trade_counterfactual_ratio < 1.0 {
        blockers
            .push("no-trade counterfactuals remain missing for some comparable rows".to_string());
        ComparableEvidenceQualityStatus::NeedMoreNoTradeCounterfactuals
    } else if config.require_risk_denied_counterfactual && risk_denied_counterfactual_ratio < 1.0 {
        blockers.push(
            "risk-denied counterfactuals remain missing for some comparable rows".to_string(),
        );
        ComparableEvidenceQualityStatus::NeedMoreRiskDeniedCounterfactuals
    } else if summary_derived_ratio > 0.50 {
        warnings.push("summary-derived evidence still dominates the comparable bundle".to_string());
        ComparableEvidenceQualityStatus::TooMuchSummaryDerived
    } else if diagnostic_only_ratio > 0.50 {
        warnings.push("diagnostic-only evidence still dominates the comparable bundle".to_string());
        ComparableEvidenceQualityStatus::TooMuchDiagnosticOnly
    } else {
        ComparableEvidenceQualityStatus::HealthyComparableEvidence
    };

    ComparableEvidenceQualityReport {
        comparable_id: bundle.comparable_id.clone(),
        total_rows,
        complete_rows,
        official_complete_rows,
        row_level_ratio,
        summary_derived_ratio,
        diagnostic_only_ratio,
        outcome_reference_ratio,
        baseline_reference_ratio,
        no_trade_counterfactual_ratio,
        risk_denied_counterfactual_ratio,
        no_lookahead_safe_ratio,
        quality_status,
        blockers,
        warnings,
        reason_codes: stable_reason_codes(
            &config
                .reason_codes
                .iter()
                .cloned()
                .chain([
                    ReasonCode::EvidenceGapDetected,
                    ReasonCode::DeterministicPath,
                ])
                .collect::<Vec<_>>(),
        ),
    }
}

impl ComparableEvidenceQualityReport {
    pub fn to_text(&self) -> String {
        [
            format!("comparable_id={}", self.comparable_id),
            format!("total_rows={}", self.total_rows),
            format!("complete_rows={}", self.complete_rows),
            format!("official_complete_rows={}", self.official_complete_rows),
            format!("row_level_ratio={:.6}", self.row_level_ratio),
            format!("summary_derived_ratio={:.6}", self.summary_derived_ratio),
            format!("diagnostic_only_ratio={:.6}", self.diagnostic_only_ratio),
            format!(
                "outcome_reference_ratio={:.6}",
                self.outcome_reference_ratio
            ),
            format!(
                "baseline_reference_ratio={:.6}",
                self.baseline_reference_ratio
            ),
            format!(
                "no_trade_counterfactual_ratio={:.6}",
                self.no_trade_counterfactual_ratio
            ),
            format!(
                "risk_denied_counterfactual_ratio={:.6}",
                self.risk_denied_counterfactual_ratio
            ),
            format!(
                "no_lookahead_safe_ratio={:.6}",
                self.no_lookahead_safe_ratio
            ),
            format!("quality_status={:?}", self.quality_status),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }
}

fn ratio(count: usize, total: usize) -> f64 {
    count as f64 / total.max(1) as f64
}

fn only_source_class(
    bundle: &ComparableCommitteeEvidenceBundle,
    class: ComparableEvidenceSourceClass,
) -> bool {
    !bundle.rows.is_empty() && bundle.rows.iter().all(|row| row.source_class == class)
}
