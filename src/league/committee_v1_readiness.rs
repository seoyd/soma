use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::chair_calibration::{ChairCalibrationRecommendation, ChairCalibrationReport};
use super::committee_decision_quality::{
    CommitteeDecisionQualityReport, CommitteeDecisionQualityStatus,
};
use super::committee_evidence_quality::{
    CommitteeEvidenceQualityReport, CommitteeEvidenceQualityStatus,
};
use super::persona_conflict_matrix::{PersonaConflictMatrix, PersonaConflictStatus};
use super::risk_calibration::{RiskCalibrationRecommendation, RiskCalibrationReport};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeV1ReadinessStatus {
    ReadyForMoreEvidence,
    ReadyForCommitteeBenchmark,
    ReadyForChairTuning,
    ReadyForRiskReview,
    ReadyForSixPersonaDesignReviewOnly,
    NotReadyEvidenceTooWeak,
    NotReadyResearchOnly,
    NotReadyFixtureOnly,
    NotReadyRiskUnstable,
    NotReadyGroupthink,
    NotReadyTooFewSamples,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeV1NextRecommendation {
    MoreOfficialCommitteeEvidence,
    KeepTrinity,
    ImproveChairFirst,
    ImprovePersonaScoringFirst,
    ImproveRiskGovernorFirst,
    RunCommitteeBenchmark,
    SixPersonaDesignReviewOnly,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeV1ReadinessReport {
    pub status: CommitteeV1ReadinessStatus,
    pub evidence_quality_status: CommitteeEvidenceQualityStatus,
    pub decision_quality_status: CommitteeDecisionQualityStatus,
    pub chair_status: String,
    pub risk_status: String,
    pub conflict_status: PersonaConflictStatus,
    pub sample_count: usize,
    pub official_sample_count: usize,
    pub research_only_ratio: f64,
    pub fixture_ratio: f64,
    pub groupthink_ratio: f64,
    pub risk_denial_ratio: f64,
    pub enough_samples: bool,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub next_recommendation: CommitteeV1NextRecommendation,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_committee_v1_readiness_report(
    evidence_quality_report: &CommitteeEvidenceQualityReport,
    decision_quality_report: &CommitteeDecisionQualityReport,
    chair_calibration_report: &ChairCalibrationReport,
    risk_calibration_report: &RiskCalibrationReport,
    conflict_matrix: &PersonaConflictMatrix,
) -> CommitteeV1ReadinessReport {
    let sample_count = decision_quality_report.decision_count;
    let scenario_count = evidence_quality_report.scenario_count.max(1);
    let research_only_ratio =
        evidence_quality_report.yfinance_research_count as f64 / scenario_count as f64;
    let fixture_ratio = evidence_quality_report.fixture_count as f64 / scenario_count as f64;
    let groupthink_ratio = decision_quality_report.groupthink_warning_ratio;
    let risk_denial_ratio = decision_quality_report.risk_denial_ratio;
    let enough_samples = sample_count >= 5;
    let mut blockers = Vec::new();
    let mut warnings = evidence_quality_report.warnings.clone();

    let status = if fixture_ratio >= 0.999 {
        blockers.push("fixture-only evidence cannot pass Committee V1 readiness".to_string());
        CommitteeV1ReadinessStatus::NotReadyFixtureOnly
    } else if evidence_quality_report.quality_status
        == CommitteeEvidenceQualityStatus::ResearchOnlyEvidence
    {
        blockers
            .push("research-only evidence cannot pass official Committee V1 readiness".to_string());
        CommitteeV1ReadinessStatus::NotReadyResearchOnly
    } else if sample_count == 0 || !enough_samples {
        blockers.push("not enough committee samples".to_string());
        CommitteeV1ReadinessStatus::NotReadyTooFewSamples
    } else if matches!(
        evidence_quality_report.quality_status,
        CommitteeEvidenceQualityStatus::InsufficientEvidence
            | CommitteeEvidenceQualityStatus::LowQualityEvidence
    ) {
        blockers.push("evidence quality is too weak for Committee V1".to_string());
        CommitteeV1ReadinessStatus::NotReadyEvidenceTooWeak
    } else if groupthink_ratio >= 0.60
        || conflict_matrix.conflict_status == PersonaConflictStatus::TooAligned
    {
        blockers.push("groupthink is too high for stable Committee V1 interpretation".to_string());
        CommitteeV1ReadinessStatus::NotReadyGroupthink
    } else if risk_calibration_report.underblocking_suspected
        || matches!(
            risk_calibration_report.final_recommendation,
            RiskCalibrationRecommendation::TightenRiskRules
        )
        || decision_quality_report.emergency_stop_ratio >= 0.34
    {
        blockers.push("risk behavior looks unstable and needs review".to_string());
        CommitteeV1ReadinessStatus::NotReadyRiskUnstable
    } else if evidence_quality_report.enough_for_design_review
        && research_only_ratio <= 0.40
        && groupthink_ratio < 0.60
        && !risk_calibration_report.overblocking_suspected
    {
        CommitteeV1ReadinessStatus::ReadyForSixPersonaDesignReviewOnly
    } else if evidence_quality_report.official_count >= 5
        && decision_quality_report.quality_status
            == CommitteeDecisionQualityStatus::HealthyResearchMvp
    {
        CommitteeV1ReadinessStatus::ReadyForCommitteeBenchmark
    } else if chair_calibration_report.final_recommendation
        != ChairCalibrationRecommendation::KeepChairV0
    {
        warnings.push(
            "chair tuning could improve committee quality before broader benchmarking".to_string(),
        );
        CommitteeV1ReadinessStatus::ReadyForChairTuning
    } else if risk_calibration_report.final_recommendation
        != RiskCalibrationRecommendation::KeepRiskGovernor
    {
        warnings.push(
            "risk review is still warranted before broader committee benchmarking".to_string(),
        );
        CommitteeV1ReadinessStatus::ReadyForRiskReview
    } else {
        CommitteeV1ReadinessStatus::ReadyForMoreEvidence
    };

    if decision_quality_report.quality_status == CommitteeDecisionQualityStatus::TooMuchDisagreement
    {
        warnings.push("persona disagreement remains high".to_string());
    }
    warnings.sort();
    warnings.dedup();

    let next_recommendation = match status {
        CommitteeV1ReadinessStatus::ReadyForCommitteeBenchmark => {
            CommitteeV1NextRecommendation::RunCommitteeBenchmark
        }
        CommitteeV1ReadinessStatus::ReadyForSixPersonaDesignReviewOnly => {
            CommitteeV1NextRecommendation::SixPersonaDesignReviewOnly
        }
        CommitteeV1ReadinessStatus::ReadyForChairTuning
        | CommitteeV1ReadinessStatus::NotReadyGroupthink => {
            CommitteeV1NextRecommendation::ImproveChairFirst
        }
        CommitteeV1ReadinessStatus::ReadyForRiskReview
        | CommitteeV1ReadinessStatus::NotReadyRiskUnstable => {
            CommitteeV1NextRecommendation::ImproveRiskGovernorFirst
        }
        CommitteeV1ReadinessStatus::NotReadyTooFewSamples => {
            CommitteeV1NextRecommendation::NeedMoreEvidence
        }
        CommitteeV1ReadinessStatus::NotReadyFixtureOnly
        | CommitteeV1ReadinessStatus::NotReadyResearchOnly
        | CommitteeV1ReadinessStatus::NotReadyEvidenceTooWeak => {
            if evidence_quality_report.official_count == 0 {
                CommitteeV1NextRecommendation::MoreOfficialCommitteeEvidence
            } else {
                CommitteeV1NextRecommendation::NeedMoreEvidence
            }
        }
        CommitteeV1ReadinessStatus::ReadyForMoreEvidence => {
            if decision_quality_report.quality_status
                == CommitteeDecisionQualityStatus::TooMuchDisagreement
            {
                CommitteeV1NextRecommendation::ImprovePersonaScoringFirst
            } else if evidence_quality_report.official_count < 5 {
                CommitteeV1NextRecommendation::MoreOfficialCommitteeEvidence
            } else {
                CommitteeV1NextRecommendation::KeepTrinity
            }
        }
    };

    CommitteeV1ReadinessReport {
        status,
        evidence_quality_status: evidence_quality_report.quality_status,
        decision_quality_status: decision_quality_report.quality_status,
        chair_status: format!("{:?}", chair_calibration_report.final_recommendation),
        risk_status: format!("{:?}", risk_calibration_report.final_recommendation),
        conflict_status: conflict_matrix.conflict_status,
        sample_count,
        official_sample_count: evidence_quality_report.official_count,
        research_only_ratio,
        fixture_ratio,
        groupthink_ratio,
        risk_denial_ratio,
        enough_samples,
        blockers,
        warnings,
        next_recommendation,
        reason_codes: vec![ReasonCode::CommitteeV1ReadinessBuilt],
    }
}

impl CommitteeV1ReadinessReport {
    pub fn to_text(&self) -> String {
        [
            format!("status={:?}", self.status),
            format!("next_recommendation={:?}", self.next_recommendation),
            format!("evidence_quality_status={:?}", self.evidence_quality_status),
            format!("decision_quality_status={:?}", self.decision_quality_status),
            format!("chair_status={}", self.chair_status),
            format!("risk_status={}", self.risk_status),
            format!("conflict_status={:?}", self.conflict_status),
            format!("sample_count={}", self.sample_count),
            format!("official_sample_count={}", self.official_sample_count),
            format!("research_only_ratio={:.6}", self.research_only_ratio),
            format!("fixture_ratio={:.6}", self.fixture_ratio),
            format!("groupthink_ratio={:.6}", self.groupthink_ratio),
            format!("risk_denial_ratio={:.6}", self.risk_denial_ratio),
            format!("enough_samples={}", self.enough_samples),
            format!("blockers={}", self.blockers.join("|")),
            format!("warnings={}", self.warnings.join("|")),
        ]
        .join("\n")
    }
}
