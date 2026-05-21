use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::committee_diagnostics::{
    CommitteeDiagnosticsAggregate, CommitteeDiagnosticsRecommendation, CommitteeDiagnosticsStatus,
};
use super::committee_evidence_quality::CommitteeEvidenceQualityStatus;
use super::persona_conflict_matrix::PersonaConflictStatus;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SixPersonaDesignReadinessConfig {
    pub min_official_scenarios: usize,
    pub min_total_scenarios: usize,
    pub min_disagreement_rate: f64,
    pub max_groupthink_frequency: f64,
    pub max_risk_overblocking_rate: f64,
    pub max_research_only_ratio: f64,
    pub require_no_live_paths: bool,
    pub require_core_check_pass: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SixPersonaDesignReadinessReport {
    pub ready_for_design_review: bool,
    pub not_ready_reasons: Vec<String>,
    pub evidence_quality_status: CommitteeEvidenceQualityStatus,
    pub conflict_status: PersonaConflictStatus,
    pub chair_status: String,
    pub risk_status: String,
    pub sample_count: usize,
    pub recommendation: SixPersonaDesignRecommendation,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SixPersonaDesignRecommendation {
    NotReady,
    KeepTrinity,
    ImproveChairFirst,
    ImproveRiskGovernorFirst,
    ImproveEvidenceFirst,
    SixPersonaDesignReviewOnly,
}

impl Default for SixPersonaDesignReadinessConfig {
    fn default() -> Self {
        Self {
            min_official_scenarios: 5,
            min_total_scenarios: 8,
            min_disagreement_rate: 0.15,
            max_groupthink_frequency: 0.60,
            max_risk_overblocking_rate: 0.80,
            max_research_only_ratio: 0.40,
            require_no_live_paths: true,
            require_core_check_pass: true,
            reason_codes: vec![ReasonCode::SixPersonaDesignReadinessBuilt],
        }
    }
}

pub fn evaluate_six_persona_design_readiness(
    aggregate: &CommitteeDiagnosticsAggregate,
    config: &SixPersonaDesignReadinessConfig,
) -> SixPersonaDesignReadinessReport {
    let mut not_ready_reasons = Vec::new();
    if aggregate.evidence_quality_report.official_count < config.min_official_scenarios {
        not_ready_reasons.push("not enough official scenarios".to_string());
    }
    if aggregate.replay_report.record_count < config.min_total_scenarios {
        not_ready_reasons.push("not enough total scenarios".to_string());
    }
    if aggregate.conflict_matrix.average_disagreement < config.min_disagreement_rate {
        not_ready_reasons.push("disagreement too low".to_string());
    }
    if aggregate.conflict_matrix.groupthink_frequency > config.max_groupthink_frequency {
        not_ready_reasons.push("groupthink too high".to_string());
    }
    let risk_overblocking_rate = aggregate
        .risk_diagnostics
        .iter()
        .filter(|report| report.veto_applied)
        .count() as f64
        / aggregate.risk_diagnostics.len().max(1) as f64;
    if risk_overblocking_rate > config.max_risk_overblocking_rate {
        not_ready_reasons.push("risk overblocking too high".to_string());
    }
    let research_ratio = aggregate.evidence_quality_report.yfinance_research_count as f64
        / aggregate.evidence_quality_report.scenario_count.max(1) as f64;
    if research_ratio > config.max_research_only_ratio {
        not_ready_reasons.push("research-only ratio too high".to_string());
    }
    if config.require_core_check_pass
        && matches!(
            aggregate.final_status,
            CommitteeDiagnosticsStatus::EvidenceTooWeak | CommitteeDiagnosticsStatus::ResearchOnly
        )
    {
        not_ready_reasons.push("core/evidence gate not strong enough".to_string());
    }
    let ready_for_design_review = not_ready_reasons.is_empty()
        && aggregate.evidence_quality_report.enough_for_design_review
        && aggregate.final_status == CommitteeDiagnosticsStatus::DiagnosticsHealthy;
    let recommendation = if ready_for_design_review {
        SixPersonaDesignRecommendation::SixPersonaDesignReviewOnly
    } else {
        match aggregate.recommendation {
            CommitteeDiagnosticsRecommendation::ImproveChairFirst => {
                SixPersonaDesignRecommendation::ImproveChairFirst
            }
            CommitteeDiagnosticsRecommendation::ImproveRiskGovernorFirst => {
                SixPersonaDesignRecommendation::ImproveRiskGovernorFirst
            }
            CommitteeDiagnosticsRecommendation::ImproveEvidenceIngestionFirst
            | CommitteeDiagnosticsRecommendation::NeedMoreEvidence => {
                SixPersonaDesignRecommendation::ImproveEvidenceFirst
            }
            _ => SixPersonaDesignRecommendation::KeepTrinity,
        }
    };
    SixPersonaDesignReadinessReport {
        ready_for_design_review,
        not_ready_reasons,
        evidence_quality_status: aggregate.evidence_quality_report.quality_status,
        conflict_status: aggregate.conflict_matrix.conflict_status,
        chair_status: format!("{:?}", aggregate.final_status),
        risk_status: aggregate
            .risk_diagnostics
            .first()
            .map(|report| format!("{:?}", report.diagnostic_status))
            .unwrap_or_else(|| "NoRiskDiagnostics".to_string()),
        sample_count: aggregate.replay_report.record_count,
        recommendation,
        reason_codes: vec![ReasonCode::SixPersonaDesignReadinessBuilt],
    }
}

impl SixPersonaDesignReadinessReport {
    pub fn to_text(&self) -> String {
        [
            format!("ready_for_design_review={}", self.ready_for_design_review),
            format!("sample_count={}", self.sample_count),
            format!("recommendation={:?}", self.recommendation),
            format!("evidence_quality_status={:?}", self.evidence_quality_status),
            format!("conflict_status={:?}", self.conflict_status),
            format!("chair_status={}", self.chair_status),
            format!("risk_status={}", self.risk_status),
            format!("not_ready_reasons={}", self.not_ready_reasons.join("|")),
        ]
        .join("\n")
    }
}
