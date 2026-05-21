use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::committee_outcome_linker::OutcomeLinkedCommitteeScenarioPack;
use super::official_committee_pack::OfficialCommitteeScenarioPack;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfficialCommitteeEvidenceReadinessStatus {
    ReadyForOfficialCommitteeBenchmark,
    ReadyForMoreOfficialEvidence,
    ReadyForOutcomeLinking,
    NotReadyResearchOnly,
    NotReadyFixtureOnly,
    NotReadyCryptoOnly,
    NotReadySummaryDerivedDominant,
    NotReadyNoOutcomeLinks,
    NotReadyNoLookaheadViolation,
    NotReadyInsufficientRows,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialCommitteeEvidenceReadinessReport {
    pub official_row_count: usize,
    pub outcome_linked_row_count: usize,
    pub baseline_linked_row_count: usize,
    pub no_trade_counterfactual_count: usize,
    pub risk_denial_counterfactual_count: usize,
    pub row_level_ratio: f64,
    pub summary_derived_ratio: f64,
    pub research_only_ratio: f64,
    pub fixture_ratio: f64,
    pub crypto_only_ratio: f64,
    pub no_lookahead_safe: bool,
    pub enough_for_committee_benchmark: bool,
    pub enough_for_six_person_design_review: bool,
    pub readiness_status: OfficialCommitteeEvidenceReadinessStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[allow(clippy::too_many_arguments)]
pub fn build_official_committee_evidence_readiness_report(
    pack: &OfficialCommitteeScenarioPack,
    linked_pack: Option<&OutcomeLinkedCommitteeScenarioPack>,
    min_official_rows: usize,
    min_outcome_linked_rows: usize,
    min_baseline_linked_rows: usize,
    min_no_trade_counterfactuals: usize,
    min_risk_denial_counterfactuals: usize,
    max_summary_derived_ratio: f64,
    max_research_only_ratio: f64,
    max_fixture_ratio: f64,
) -> OfficialCommitteeEvidenceReadinessReport {
    let outcome_linked_row_count = linked_pack
        .map(|linked_pack| linked_pack.outcome_linked_count)
        .unwrap_or(pack.outcome_linked_count);
    let baseline_linked_row_count = linked_pack
        .map(|linked_pack| linked_pack.baseline_linked_count)
        .unwrap_or(pack.baseline_reference_count);
    let no_trade_counterfactual_count = linked_pack
        .map(|linked_pack| linked_pack.no_trade_counterfactual_count)
        .unwrap_or(pack.no_trade_counterfactual_count);
    let risk_denial_counterfactual_count = linked_pack
        .map(|linked_pack| linked_pack.risk_denial_counterfactual_count)
        .unwrap_or(pack.risk_denial_counterfactual_count);
    let no_lookahead_safe = linked_pack
        .map(|linked_pack| linked_pack.no_lookahead_violations == 0)
        .unwrap_or(true);
    let enough_for_committee_benchmark = pack.official_row_count >= min_official_rows
        && outcome_linked_row_count >= min_outcome_linked_rows
        && baseline_linked_row_count >= min_baseline_linked_rows
        && no_trade_counterfactual_count >= min_no_trade_counterfactuals
        && risk_denial_counterfactual_count >= min_risk_denial_counterfactuals
        && pack.summary_derived_ratio() <= max_summary_derived_ratio
        && pack.research_only_ratio() <= max_research_only_ratio
        && pack.fixture_ratio() <= max_fixture_ratio
        && no_lookahead_safe;
    let enough_for_six_person_design_review = enough_for_committee_benchmark
        && pack.official_row_count
            >= min_official_rows
                .saturating_mul(2)
                .max(min_official_rows + 3)
        && outcome_linked_row_count >= min_outcome_linked_rows.saturating_mul(2)
        && baseline_linked_row_count >= min_baseline_linked_rows.saturating_mul(2)
        && pack.summary_derived_ratio() <= (max_summary_derived_ratio * 0.75).max(0.10)
        && pack.crypto_only_ratio() < 0.999;
    let readiness_status = if pack.fixture_ratio() >= 0.999 && pack.row_count() > 0 {
        OfficialCommitteeEvidenceReadinessStatus::NotReadyFixtureOnly
    } else if pack.research_only_ratio() >= 0.999 && pack.row_count() > 0 {
        OfficialCommitteeEvidenceReadinessStatus::NotReadyResearchOnly
    } else if pack.crypto_only_ratio() >= 0.999 && pack.row_count() > 0 {
        OfficialCommitteeEvidenceReadinessStatus::NotReadyCryptoOnly
    } else if pack.official_row_count < min_official_rows {
        OfficialCommitteeEvidenceReadinessStatus::NotReadyInsufficientRows
    } else if !no_lookahead_safe {
        OfficialCommitteeEvidenceReadinessStatus::NotReadyNoLookaheadViolation
    } else if pack.summary_derived_ratio() > max_summary_derived_ratio {
        OfficialCommitteeEvidenceReadinessStatus::NotReadySummaryDerivedDominant
    } else if outcome_linked_row_count == 0 {
        OfficialCommitteeEvidenceReadinessStatus::ReadyForOutcomeLinking
    } else if outcome_linked_row_count < min_outcome_linked_rows {
        OfficialCommitteeEvidenceReadinessStatus::NotReadyNoOutcomeLinks
    } else if enough_for_committee_benchmark {
        OfficialCommitteeEvidenceReadinessStatus::ReadyForOfficialCommitteeBenchmark
    } else {
        OfficialCommitteeEvidenceReadinessStatus::ReadyForMoreOfficialEvidence
    };
    OfficialCommitteeEvidenceReadinessReport {
        official_row_count: pack.official_row_count,
        outcome_linked_row_count,
        baseline_linked_row_count,
        no_trade_counterfactual_count,
        risk_denial_counterfactual_count,
        row_level_ratio: pack.row_level_ratio(),
        summary_derived_ratio: pack.summary_derived_ratio(),
        research_only_ratio: pack.research_only_ratio(),
        fixture_ratio: pack.fixture_ratio(),
        crypto_only_ratio: pack.crypto_only_ratio(),
        no_lookahead_safe,
        enough_for_committee_benchmark,
        enough_for_six_person_design_review,
        readiness_status,
        reason_codes: vec![ReasonCode::OfficialCommitteeReadinessBuilt],
    }
}

impl OfficialCommitteeEvidenceReadinessReport {
    pub fn to_text(&self) -> String {
        [
            format!("official_row_count={}", self.official_row_count),
            format!("outcome_linked_row_count={}", self.outcome_linked_row_count),
            format!(
                "baseline_linked_row_count={}",
                self.baseline_linked_row_count
            ),
            format!(
                "no_trade_counterfactual_count={}",
                self.no_trade_counterfactual_count
            ),
            format!(
                "risk_denial_counterfactual_count={}",
                self.risk_denial_counterfactual_count
            ),
            format!("row_level_ratio={:.6}", self.row_level_ratio),
            format!("summary_derived_ratio={:.6}", self.summary_derived_ratio),
            format!("research_only_ratio={:.6}", self.research_only_ratio),
            format!("fixture_ratio={:.6}", self.fixture_ratio),
            format!("crypto_only_ratio={:.6}", self.crypto_only_ratio),
            format!("no_lookahead_safe={}", self.no_lookahead_safe),
            format!(
                "enough_for_committee_benchmark={}",
                self.enough_for_committee_benchmark
            ),
            format!(
                "enough_for_six_person_design_review={}",
                self.enough_for_six_person_design_review
            ),
            format!("readiness_status={:?}", self.readiness_status),
        ]
        .join("\n")
    }
}
