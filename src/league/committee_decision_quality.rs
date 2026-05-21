use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::chair_diagnostics::{ChairDiagnosticStatus, ChairDiagnosticsReport};
use super::committee_decision::ChairCommitteeConfig;
use super::committee_evidence_quality::{
    CommitteeEvidenceQualityReport, CommitteeEvidenceQualityStatus,
};
use super::committee_replay::CommitteeReplayReport;
use super::committee_risk_bridge::CommitteeFinalAction;
use super::persona_conflict_matrix::PersonaConflictMatrix;
use super::risk_bridge_diagnostics::{RiskBridgeDiagnosticStatus, RiskBridgeDiagnosticsReport};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeDecisionQualityStatus {
    HealthyResearchMvp,
    AllNoTrade,
    RiskBlockedDominant,
    TooMuchGroupthink,
    TooMuchDisagreement,
    EvidenceTooWeak,
    ResearchOnly,
    FixtureOnly,
    InsufficientSamples,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeDecisionQualityReport {
    pub decision_count: usize,
    pub source_summary: String,
    pub final_action_counts: BTreeMap<String, usize>,
    pub chair_decision_counts: BTreeMap<String, usize>,
    pub persona_stance_counts: BTreeMap<String, usize>,
    pub no_trade_ratio: f64,
    pub approve_candidate_ratio: f64,
    pub reduce_size_ratio: f64,
    pub require_confirm_ratio: f64,
    pub risk_denial_ratio: f64,
    pub hard_veto_ratio: f64,
    pub emergency_stop_ratio: f64,
    pub cooldown_ratio: f64,
    pub groupthink_warning_ratio: f64,
    pub high_disagreement_ratio: f64,
    pub average_disagreement: f64,
    pub average_uncertainty: f64,
    pub average_weighted_score: f64,
    pub average_expected_edge_after_cost: f64,
    pub average_expected_drawdown: f64,
    pub data_quality_distribution: BTreeMap<String, usize>,
    pub evidence_quality_status: CommitteeEvidenceQualityStatus,
    pub quality_status: CommitteeDecisionQualityStatus,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_committee_decision_quality_report(
    replay_report: &CommitteeReplayReport,
    chair_diagnostics: &[ChairDiagnosticsReport],
    risk_diagnostics: &[RiskBridgeDiagnosticsReport],
    conflict_matrix: &PersonaConflictMatrix,
    evidence_quality_report: &CommitteeEvidenceQualityReport,
) -> CommitteeDecisionQualityReport {
    let decision_count = replay_report.record_count;
    let mut persona_stance_counts = BTreeMap::new();
    let mut data_quality_distribution = BTreeMap::new();
    let mut expected_edge_sum = 0.0;
    let mut expected_drawdown_sum = 0.0;
    for record in &replay_report.records {
        for vote in &record.persona_votes {
            *persona_stance_counts
                .entry(format!("{:?}", vote.stance))
                .or_insert(0) += 1;
        }
        *data_quality_distribution
            .entry(data_quality_bucket(record.scenario_row.data_quality_score))
            .or_insert(0) += 1;
        expected_edge_sum += record.scenario_row.expected_edge_after_cost;
        expected_drawdown_sum += record.scenario_row.expected_drawdown;
    }
    let chair_thresholds = ChairCommitteeConfig::default();
    let denominator = decision_count.max(1) as f64;
    let no_trade_ratio = ratio(
        replay_report
            .final_action_counts
            .get(&format!("{:?}", CommitteeFinalAction::FinalNoTrade))
            .copied()
            .unwrap_or(0),
        decision_count,
    );
    let risk_denial_ratio = ratio(
        replay_report
            .final_action_counts
            .get(&format!("{:?}", CommitteeFinalAction::FinalDenied))
            .copied()
            .unwrap_or(0),
        decision_count,
    );
    let approve_candidate_ratio = ratio(
        replay_report
            .chair_decision_counts
            .get("ApproveCandidate")
            .copied()
            .unwrap_or(0),
        decision_count,
    );
    let reduce_size_ratio = ratio(
        replay_report
            .chair_decision_counts
            .get("ReduceSizeCandidate")
            .copied()
            .unwrap_or(0),
        decision_count,
    );
    let require_confirm_ratio = ratio(
        replay_report
            .chair_decision_counts
            .get("RequireHumanConfirm")
            .copied()
            .unwrap_or(0),
        decision_count,
    );
    let hard_veto_ratio = ratio(
        replay_report
            .chair_decision_counts
            .get("Vetoed")
            .copied()
            .unwrap_or(0),
        decision_count,
    );
    let emergency_stop_ratio = ratio(
        risk_diagnostics
            .iter()
            .filter(|report| report.diagnostic_status == RiskBridgeDiagnosticStatus::EmergencyStop)
            .count(),
        decision_count,
    );
    let cooldown_ratio = ratio(
        risk_diagnostics
            .iter()
            .filter(|report| report.diagnostic_status == RiskBridgeDiagnosticStatus::Cooldown)
            .count(),
        decision_count,
    );
    let groupthink_warning_ratio = ratio(
        chair_diagnostics
            .iter()
            .filter(|report| {
                report.groupthink_risk >= chair_thresholds.groupthink_warning_threshold
                    || report.diagnostic_status == ChairDiagnosticStatus::GroupthinkRisk
            })
            .count(),
        decision_count,
    );
    let high_disagreement_ratio = ratio(
        chair_diagnostics
            .iter()
            .filter(|report| {
                report.disagreement_score >= 0.55
                    || report.diagnostic_status == ChairDiagnosticStatus::ExcessiveDisagreement
            })
            .count(),
        decision_count,
    );
    let average_disagreement = chair_diagnostics
        .iter()
        .map(|report| report.disagreement_score)
        .sum::<f64>()
        / denominator;
    let average_uncertainty = chair_diagnostics
        .iter()
        .map(|report| report.uncertainty)
        .sum::<f64>()
        / denominator;
    let average_weighted_score = chair_diagnostics
        .iter()
        .map(|report| report.weighted_score)
        .sum::<f64>()
        / denominator;
    let quality_status = if decision_count == 0 || decision_count < 3 {
        CommitteeDecisionQualityStatus::InsufficientSamples
    } else if evidence_quality_report.quality_status
        == CommitteeEvidenceQualityStatus::FixtureOnlyEvidence
    {
        CommitteeDecisionQualityStatus::FixtureOnly
    } else if evidence_quality_report.quality_status
        == CommitteeEvidenceQualityStatus::ResearchOnlyEvidence
    {
        CommitteeDecisionQualityStatus::ResearchOnly
    } else if matches!(
        evidence_quality_report.quality_status,
        CommitteeEvidenceQualityStatus::InsufficientEvidence
            | CommitteeEvidenceQualityStatus::LowQualityEvidence
    ) {
        CommitteeDecisionQualityStatus::EvidenceTooWeak
    } else if no_trade_ratio >= 0.999 {
        CommitteeDecisionQualityStatus::AllNoTrade
    } else if risk_denial_ratio >= 0.75 {
        CommitteeDecisionQualityStatus::RiskBlockedDominant
    } else if groupthink_warning_ratio >= 0.60 || conflict_matrix.groupthink_frequency >= 0.60 {
        CommitteeDecisionQualityStatus::TooMuchGroupthink
    } else if high_disagreement_ratio >= 0.60 || conflict_matrix.average_disagreement >= 0.75 {
        CommitteeDecisionQualityStatus::TooMuchDisagreement
    } else {
        CommitteeDecisionQualityStatus::HealthyResearchMvp
    };
    CommitteeDecisionQualityReport {
        decision_count,
        source_summary: replay_report.source_summary.clone(),
        final_action_counts: replay_report.final_action_counts.clone(),
        chair_decision_counts: replay_report.chair_decision_counts.clone(),
        persona_stance_counts,
        no_trade_ratio,
        approve_candidate_ratio,
        reduce_size_ratio,
        require_confirm_ratio,
        risk_denial_ratio,
        hard_veto_ratio,
        emergency_stop_ratio,
        cooldown_ratio,
        groupthink_warning_ratio,
        high_disagreement_ratio,
        average_disagreement,
        average_uncertainty,
        average_weighted_score,
        average_expected_edge_after_cost: expected_edge_sum / denominator,
        average_expected_drawdown: expected_drawdown_sum / denominator,
        data_quality_distribution,
        evidence_quality_status: evidence_quality_report.quality_status,
        quality_status,
        reason_codes: vec![ReasonCode::CommitteeDecisionQualityBuilt],
    }
}

impl CommitteeDecisionQualityReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("decision_count={}", self.decision_count),
            format!("source_summary={}", self.source_summary),
            format!("evidence_quality_status={:?}", self.evidence_quality_status),
            format!("quality_status={:?}", self.quality_status),
            format!("no_trade_ratio={:.6}", self.no_trade_ratio),
            format!(
                "approve_candidate_ratio={:.6}",
                self.approve_candidate_ratio
            ),
            format!("reduce_size_ratio={:.6}", self.reduce_size_ratio),
            format!("require_confirm_ratio={:.6}", self.require_confirm_ratio),
            format!("risk_denial_ratio={:.6}", self.risk_denial_ratio),
            format!("hard_veto_ratio={:.6}", self.hard_veto_ratio),
            format!("emergency_stop_ratio={:.6}", self.emergency_stop_ratio),
            format!("cooldown_ratio={:.6}", self.cooldown_ratio),
            format!(
                "groupthink_warning_ratio={:.6}",
                self.groupthink_warning_ratio
            ),
            format!(
                "high_disagreement_ratio={:.6}",
                self.high_disagreement_ratio
            ),
            format!("average_disagreement={:.6}", self.average_disagreement),
            format!("average_uncertainty={:.6}", self.average_uncertainty),
            format!("average_weighted_score={:.6}", self.average_weighted_score),
            format!(
                "average_expected_edge_after_cost={:.6}",
                self.average_expected_edge_after_cost
            ),
            format!(
                "average_expected_drawdown={:.6}",
                self.average_expected_drawdown
            ),
        ];
        for (bucket, count) in &self.data_quality_distribution {
            lines.push(format!("data_quality_bucket={bucket};count={count}"));
        }
        for (stance, count) in &self.persona_stance_counts {
            lines.push(format!("persona_stance={stance};count={count}"));
        }
        lines.join("\n")
    }
}

fn ratio(count: usize, total: usize) -> f64 {
    count as f64 / total.max(1) as f64
}

fn data_quality_bucket(score: f64) -> String {
    if score < 0.80 {
        "lt_0_80".to_string()
    } else if score < 0.90 {
        "0_80_to_0_89".to_string()
    } else {
        "ge_0_90".to_string()
    }
}
