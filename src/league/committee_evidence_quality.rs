use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::EvidenceSourceKind;

use super::committee_scenario_loader::{CommitteeScenarioSet, CommitteeScenarioSourceKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeEvidenceQualityStatus {
    OfficialEvidenceAvailable,
    CryptoOnlyEvidence,
    ResearchOnlyEvidence,
    FixtureOnlyEvidence,
    InsufficientEvidence,
    LowQualityEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeEvidenceQualityReport {
    pub source_summary: String,
    pub official_count: usize,
    pub crypto_only_count: usize,
    pub yfinance_research_count: usize,
    pub fixture_count: usize,
    pub synthetic_test_count: usize,
    pub missing_provenance_count: usize,
    pub low_quality_count: usize,
    pub scenario_count: usize,
    pub enough_for_design_review: bool,
    pub quality_status: CommitteeEvidenceQualityStatus,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_committee_evidence_quality_report(
    scenario_set: &CommitteeScenarioSet,
) -> CommitteeEvidenceQualityReport {
    let official_count = scenario_set
        .rows
        .iter()
        .filter(|row| {
            row.evidence_source_kind.readiness_eligible()
                && !matches!(
                    row.source_kind,
                    CommitteeScenarioSourceKind::Fixture
                        | CommitteeScenarioSourceKind::SyntheticTest
                )
        })
        .count();
    let crypto_only_count = scenario_set
        .rows
        .iter()
        .filter(|row| row.market == crate::data::ProviderMarket::Crypto)
        .count();
    let yfinance_research_count = scenario_set
        .rows
        .iter()
        .filter(|row| row.evidence_source_kind == EvidenceSourceKind::YFinanceResearch)
        .count();
    let fixture_count = scenario_set
        .rows
        .iter()
        .filter(|row| row.source_kind == CommitteeScenarioSourceKind::Fixture)
        .count();
    let synthetic_test_count = scenario_set
        .rows
        .iter()
        .filter(|row| row.source_kind == CommitteeScenarioSourceKind::SyntheticTest)
        .count();
    let missing_provenance_count = scenario_set
        .rows
        .iter()
        .filter(|row| {
            row.provenance_summary.trim().is_empty() || row.provenance_summary.contains("missing")
        })
        .count();
    let low_quality_count = scenario_set
        .rows
        .iter()
        .filter(|row| row.data_quality_score < 0.80)
        .count();
    let scenario_count = scenario_set.rows.len();
    let enough_for_design_review = official_count >= 5
        && yfinance_research_count * 2 <= scenario_count.max(1)
        && fixture_count == 0
        && synthetic_test_count == 0
        && missing_provenance_count == 0
        && low_quality_count == 0;
    let quality_status = if scenario_count == 0 {
        CommitteeEvidenceQualityStatus::InsufficientEvidence
    } else if official_count > 0 && low_quality_count == 0 {
        CommitteeEvidenceQualityStatus::OfficialEvidenceAvailable
    } else if fixture_count == scenario_count {
        CommitteeEvidenceQualityStatus::FixtureOnlyEvidence
    } else if yfinance_research_count == scenario_count {
        CommitteeEvidenceQualityStatus::ResearchOnlyEvidence
    } else if crypto_only_count == scenario_count {
        CommitteeEvidenceQualityStatus::CryptoOnlyEvidence
    } else if low_quality_count > 0 {
        CommitteeEvidenceQualityStatus::LowQualityEvidence
    } else {
        CommitteeEvidenceQualityStatus::InsufficientEvidence
    };
    let mut warnings = Vec::new();
    if fixture_count > 0 {
        warnings.push("fixture rows are architecture tests only".to_string());
    }
    if yfinance_research_count > 0 {
        warnings.push("yfinance rows remain research-only".to_string());
    }
    if crypto_only_count > 0 && official_count == 0 {
        warnings.push("crypto-only evidence is not equity-ready".to_string());
    }
    if missing_provenance_count > 0 {
        warnings.push("missing provenance detected".to_string());
    }
    if low_quality_count > 0 {
        warnings.push("low quality scenarios detected".to_string());
    }
    warnings.sort();
    warnings.dedup();
    CommitteeEvidenceQualityReport {
        source_summary: scenario_set.source_summary.clone(),
        official_count,
        crypto_only_count,
        yfinance_research_count,
        fixture_count,
        synthetic_test_count,
        missing_provenance_count,
        low_quality_count,
        scenario_count,
        enough_for_design_review,
        quality_status,
        warnings,
        reason_codes: vec![ReasonCode::CommitteeEvidenceQualityBuilt],
    }
}

impl CommitteeEvidenceQualityReport {
    pub fn to_text(&self) -> String {
        [
            format!("source_summary={}", self.source_summary),
            format!("quality_status={:?}", self.quality_status),
            format!("scenario_count={}", self.scenario_count),
            format!("official_count={}", self.official_count),
            format!("crypto_only_count={}", self.crypto_only_count),
            format!("yfinance_research_count={}", self.yfinance_research_count),
            format!("fixture_count={}", self.fixture_count),
            format!("synthetic_test_count={}", self.synthetic_test_count),
            format!("missing_provenance_count={}", self.missing_provenance_count),
            format!("low_quality_count={}", self.low_quality_count),
            format!("enough_for_design_review={}", self.enough_for_design_review),
            format!("warnings={}", self.warnings.join("|")),
        ]
        .join("\n")
    }
}
