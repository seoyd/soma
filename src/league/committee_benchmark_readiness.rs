use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::committee_actionability::{CommitteeActionabilityReport, CommitteeActionabilityStatus};
use super::committee_attribution::{CommitteeAttributionReport, CommitteeAttributionStatus};
use super::committee_decision_quality::{
    CommitteeDecisionQualityReport, CommitteeDecisionQualityStatus,
};
use super::committee_materialization::CommitteeMaterializationConfig;
use super::committee_scenario_loader::{
    CommitteeScenarioMaterializationLevel, CommitteeScenarioSet,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeBenchmarkReadinessStatus {
    ReadyForCommitteeBenchmark,
    ReadyForMoreOfficialEvidence,
    ReadyForChairTuning,
    ReadyForPersonaScoringTuning,
    ReadyForRiskReview,
    ReadyForSixPersonaDesignReviewOnly,
    NotReadyFixtureOnly,
    NotReadyResearchOnly,
    NotReadyCryptoOnly,
    NotReadyInsufficientRows,
    NotReadyInsufficientOutcomes,
    NotReadyMaterializationWeak,
    NotReadyRiskBlockedDominant,
    NotReadyGroupthink,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeBenchmarkNextRecommendation {
    MoreOfficialCommitteeEvidence,
    ImproveMaterializationFirst,
    ImproveChairFirst,
    ImprovePersonaScoringFirst,
    ImproveRiskGovernorFirst,
    KeepTrinity,
    SixPersonaDesignReviewOnly,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeBenchmarkReadinessReport {
    pub status: CommitteeBenchmarkReadinessStatus,
    pub official_row_count: usize,
    pub total_row_count: usize,
    pub outcome_reference_count: usize,
    pub row_level_materialization_ratio: f64,
    pub summary_derived_ratio: f64,
    pub research_only_ratio: f64,
    pub fixture_ratio: f64,
    pub crypto_only_ratio: f64,
    pub actionability_status: CommitteeActionabilityStatus,
    pub attribution_status: CommitteeAttributionStatus,
    pub decision_quality_status: CommitteeDecisionQualityStatus,
    pub risk_status: String,
    pub chair_status: String,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub next_recommendation: CommitteeBenchmarkNextRecommendation,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_committee_benchmark_readiness_report(
    scenario_set: &CommitteeScenarioSet,
    materialization_config: Option<&CommitteeMaterializationConfig>,
    decision_quality_report: &CommitteeDecisionQualityReport,
    actionability_report: &CommitteeActionabilityReport,
    attribution_report: &CommitteeAttributionReport,
    min_official_rows: usize,
    min_total_rows: usize,
    min_outcome_references: usize,
) -> CommitteeBenchmarkReadinessReport {
    let total_row_count = scenario_set.row_count;
    let official_row_count = scenario_set.official_row_count;
    let outcome_reference_count = scenario_set
        .rows
        .iter()
        .filter(|row| row.outcome_reference.is_some())
        .count();
    let row_level_count = scenario_set
        .rows
        .iter()
        .filter(|row| row.materialization_level == CommitteeScenarioMaterializationLevel::RowLevel)
        .count();
    let summary_count = scenario_set
        .rows
        .iter()
        .filter(|row| row.reason_codes.contains(&ReasonCode::SummaryDerived))
        .count();
    let research_only_ratio =
        scenario_set.research_only_row_count as f64 / total_row_count.max(1) as f64;
    let fixture_ratio = scenario_set.fixture_row_count as f64 / total_row_count.max(1) as f64;
    let crypto_only_ratio = scenario_set
        .rows
        .iter()
        .filter(|row| row.market == crate::data::ProviderMarket::Crypto)
        .count() as f64
        / total_row_count.max(1) as f64;
    let row_level_materialization_ratio = row_level_count as f64 / total_row_count.max(1) as f64;
    let summary_derived_ratio = summary_count as f64 / total_row_count.max(1) as f64;
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let status = if fixture_ratio >= 0.999 && total_row_count > 0 {
        blockers.push("fixture-only rows cannot pass benchmark readiness".to_string());
        CommitteeBenchmarkReadinessStatus::NotReadyFixtureOnly
    } else if research_only_ratio >= 0.999 && total_row_count > 0 {
        blockers.push("yfinance-only rows cannot pass official benchmark readiness".to_string());
        CommitteeBenchmarkReadinessStatus::NotReadyResearchOnly
    } else if crypto_only_ratio >= 0.999 && total_row_count > 0 {
        blockers.push(
            "crypto-only rows cannot claim cross-market committee benchmark readiness".to_string(),
        );
        CommitteeBenchmarkReadinessStatus::NotReadyCryptoOnly
    } else if total_row_count < min_total_rows {
        blockers.push("not enough materialized rows".to_string());
        CommitteeBenchmarkReadinessStatus::NotReadyInsufficientRows
    } else if outcome_reference_count < min_outcome_references {
        blockers.push("not enough outcome references".to_string());
        CommitteeBenchmarkReadinessStatus::NotReadyInsufficientOutcomes
    } else if row_level_materialization_ratio < 0.50 {
        blockers.push("row-level materialization ratio is too low".to_string());
        CommitteeBenchmarkReadinessStatus::NotReadyMaterializationWeak
    } else if matches!(
        decision_quality_report.quality_status,
        CommitteeDecisionQualityStatus::RiskBlockedDominant
            | CommitteeDecisionQualityStatus::AllNoTrade
    ) || actionability_report.actionability_status
        == CommitteeActionabilityStatus::MostlyRiskDenied
    {
        blockers.push("risk-blocked behavior dominates benchmark output".to_string());
        CommitteeBenchmarkReadinessStatus::NotReadyRiskBlockedDominant
    } else if matches!(
        decision_quality_report.quality_status,
        CommitteeDecisionQualityStatus::TooMuchGroupthink
    ) || attribution_report.attribution_status
        == CommitteeAttributionStatus::PersonaDominated
    {
        blockers.push("committee influence is too concentrated".to_string());
        CommitteeBenchmarkReadinessStatus::NotReadyGroupthink
    } else if official_row_count >= min_official_rows {
        if summary_derived_ratio <= 0.40 {
            CommitteeBenchmarkReadinessStatus::ReadyForCommitteeBenchmark
        } else {
            warnings
                .push("official rows exist but summary-derived share is still high".to_string());
            CommitteeBenchmarkReadinessStatus::ReadyForMoreOfficialEvidence
        }
    } else if decision_quality_report.quality_status
        == CommitteeDecisionQualityStatus::TooMuchDisagreement
    {
        CommitteeBenchmarkReadinessStatus::ReadyForPersonaScoringTuning
    } else if matches!(
        materialization_config,
        Some(config) if config.prefer_row_level_artifacts && config.allow_summary_derived_rows
    ) {
        CommitteeBenchmarkReadinessStatus::ReadyForMoreOfficialEvidence
    } else {
        CommitteeBenchmarkReadinessStatus::ReadyForRiskReview
    };
    let next_recommendation = match status {
        CommitteeBenchmarkReadinessStatus::ReadyForCommitteeBenchmark => {
            CommitteeBenchmarkNextRecommendation::KeepTrinity
        }
        CommitteeBenchmarkReadinessStatus::ReadyForMoreOfficialEvidence => {
            CommitteeBenchmarkNextRecommendation::MoreOfficialCommitteeEvidence
        }
        CommitteeBenchmarkReadinessStatus::ReadyForChairTuning
        | CommitteeBenchmarkReadinessStatus::NotReadyGroupthink => {
            CommitteeBenchmarkNextRecommendation::ImproveChairFirst
        }
        CommitteeBenchmarkReadinessStatus::ReadyForPersonaScoringTuning => {
            CommitteeBenchmarkNextRecommendation::ImprovePersonaScoringFirst
        }
        CommitteeBenchmarkReadinessStatus::ReadyForRiskReview
        | CommitteeBenchmarkReadinessStatus::NotReadyRiskBlockedDominant => {
            CommitteeBenchmarkNextRecommendation::ImproveRiskGovernorFirst
        }
        CommitteeBenchmarkReadinessStatus::ReadyForSixPersonaDesignReviewOnly => {
            CommitteeBenchmarkNextRecommendation::SixPersonaDesignReviewOnly
        }
        CommitteeBenchmarkReadinessStatus::NotReadyMaterializationWeak => {
            CommitteeBenchmarkNextRecommendation::ImproveMaterializationFirst
        }
        CommitteeBenchmarkReadinessStatus::NotReadyFixtureOnly
        | CommitteeBenchmarkReadinessStatus::NotReadyResearchOnly
        | CommitteeBenchmarkReadinessStatus::NotReadyCryptoOnly
        | CommitteeBenchmarkReadinessStatus::NotReadyInsufficientRows
        | CommitteeBenchmarkReadinessStatus::NotReadyInsufficientOutcomes => {
            CommitteeBenchmarkNextRecommendation::NeedMoreEvidence
        }
    };
    CommitteeBenchmarkReadinessReport {
        status,
        official_row_count,
        total_row_count,
        outcome_reference_count,
        row_level_materialization_ratio,
        summary_derived_ratio,
        research_only_ratio,
        fixture_ratio,
        crypto_only_ratio,
        actionability_status: actionability_report.actionability_status,
        attribution_status: attribution_report.attribution_status,
        decision_quality_status: decision_quality_report.quality_status,
        risk_status: format!("{:?}", actionability_report.actionability_status),
        chair_status: format!("{:?}", attribution_report.attribution_status),
        blockers,
        warnings,
        next_recommendation,
        reason_codes: vec![ReasonCode::CommitteeBenchmarkReadinessBuilt],
    }
}

impl CommitteeBenchmarkReadinessReport {
    pub fn to_text(&self) -> String {
        [
            format!("status={:?}", self.status),
            format!("official_row_count={}", self.official_row_count),
            format!("total_row_count={}", self.total_row_count),
            format!("outcome_reference_count={}", self.outcome_reference_count),
            format!(
                "row_level_materialization_ratio={:.6}",
                self.row_level_materialization_ratio
            ),
            format!("summary_derived_ratio={:.6}", self.summary_derived_ratio),
            format!("research_only_ratio={:.6}", self.research_only_ratio),
            format!("fixture_ratio={:.6}", self.fixture_ratio),
            format!("crypto_only_ratio={:.6}", self.crypto_only_ratio),
            format!("actionability_status={:?}", self.actionability_status),
            format!("attribution_status={:?}", self.attribution_status),
            format!("decision_quality_status={:?}", self.decision_quality_status),
            format!("next_recommendation={:?}", self.next_recommendation),
            format!("blockers={}", self.blockers.join("|")),
            format!("warnings={}", self.warnings.join("|")),
        ]
        .join("\n")
    }
}
