use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_ordered_strings, stable_reason_codes};
use crate::ui::{CandidatePanel, CandidateStatus};

use super::{
    HumanConfirmProtocolConfig, OwnerInput, OwnerInputKind, build_owner_review_queue,
    validate_owner_input,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerDecisionImpactKind {
    #[default]
    NoImpact,
    CandidateHeld,
    CandidateDismissed,
    ReanalysisRequested,
    PaperConfirmed,
    RiskMadeMoreConservative,
    DiagnosticOnly,
    BlockedByPolicy,
    BlockedByRiskGovernor,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerDecisionImpactRecord {
    pub owner_input_id: String,
    #[serde(default)]
    pub target_id: Option<String>,
    #[serde(default)]
    pub before_status: Option<String>,
    #[serde(default)]
    pub after_status: Option<String>,
    pub impact_kind: OwnerDecisionImpactKind,
    #[serde(default)]
    pub affected_panels: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerDecisionImpactFinalStatus {
    OwnerInputApplied,
    OwnerInputPartiallyApplied,
    OwnerInputBlocked,
    OwnerInputDiagnosticOnly,
    #[default]
    NoOwnerInputs,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerDecisionImpactReport {
    pub report_id: String,
    #[serde(default)]
    pub records: Vec<OwnerDecisionImpactRecord>,
    pub accepted_count: usize,
    pub blocked_count: usize,
    pub diagnostic_only_count: usize,
    pub paper_confirm_count: usize,
    pub dismissed_count: usize,
    pub held_count: usize,
    pub reanalysis_requested_count: usize,
    pub final_status: OwnerDecisionImpactFinalStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl OwnerDecisionImpactRecord {
    pub fn stabilize(&mut self) {
        self.affected_panels = stable_ordered_strings(&self.affected_panels);
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

impl OwnerDecisionImpactReport {
    pub fn stabilize(&mut self) {
        self.records
            .sort_by(|left, right| left.owner_input_id.cmp(&right.owner_input_id));
        for record in &mut self.records {
            record.stabilize();
        }
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }

    pub fn fingerprint(&self) -> String {
        let mut copy = self.clone();
        copy.stabilize();
        stable_hash_string(&serde_json::to_string(&copy).unwrap_or_else(|_| copy.report_id.clone()))
    }

    pub fn to_text(&self) -> String {
        [
            "research_only_warning=owner impact report is audited and local-only".to_string(),
            "paper_only_warning=paper confirm impact never means real execution".to_string(),
            format!("report_id={}", self.report_id),
            format!("accepted_count={}", self.accepted_count),
            format!("blocked_count={}", self.blocked_count),
            format!("diagnostic_only_count={}", self.diagnostic_only_count),
            format!("paper_confirm_count={}", self.paper_confirm_count),
            format!("dismissed_count={}", self.dismissed_count),
            format!("held_count={}", self.held_count),
            format!(
                "reanalysis_requested_count={}",
                self.reanalysis_requested_count
            ),
            format!("final_status={:?}", self.final_status),
            format!("fingerprint={}", self.fingerprint()),
        ]
        .join("\n")
    }
}

pub fn build_owner_decision_impact_report(
    report_id: &str,
    candidate_panel: &CandidatePanel,
    owner_inputs: &[OwnerInput],
    protocol: &HumanConfirmProtocolConfig,
) -> OwnerDecisionImpactReport {
    let queue = build_owner_review_queue(report_id, candidate_panel, owner_inputs, protocol);
    let mut report = OwnerDecisionImpactReport {
        report_id: report_id.to_string(),
        records: owner_inputs
            .iter()
            .map(|input| build_impact_record(candidate_panel, &queue, input))
            .collect(),
        accepted_count: 0,
        blocked_count: 0,
        diagnostic_only_count: 0,
        paper_confirm_count: 0,
        dismissed_count: 0,
        held_count: 0,
        reanalysis_requested_count: 0,
        final_status: OwnerDecisionImpactFinalStatus::NoOwnerInputs,
        reason_codes: vec![ReasonCode::OwnerImpactReportBuilt],
    };
    for record in &report.records {
        match record.impact_kind {
            OwnerDecisionImpactKind::BlockedByPolicy
            | OwnerDecisionImpactKind::BlockedByRiskGovernor => {
                report.blocked_count += 1;
            }
            OwnerDecisionImpactKind::DiagnosticOnly | OwnerDecisionImpactKind::NoImpact => {
                report.diagnostic_only_count += 1;
            }
            OwnerDecisionImpactKind::PaperConfirmed => {
                report.accepted_count += 1;
                report.paper_confirm_count += 1;
            }
            OwnerDecisionImpactKind::CandidateDismissed => {
                report.accepted_count += 1;
                report.dismissed_count += 1;
            }
            OwnerDecisionImpactKind::CandidateHeld => {
                report.accepted_count += 1;
                report.held_count += 1;
            }
            OwnerDecisionImpactKind::ReanalysisRequested => {
                report.accepted_count += 1;
                report.reanalysis_requested_count += 1;
            }
            OwnerDecisionImpactKind::RiskMadeMoreConservative => {
                report.accepted_count += 1;
            }
        }
    }
    report.final_status = if report.records.is_empty() {
        OwnerDecisionImpactFinalStatus::NoOwnerInputs
    } else if report.accepted_count > 0
        && report.blocked_count == 0
        && report.diagnostic_only_count == 0
    {
        OwnerDecisionImpactFinalStatus::OwnerInputApplied
    } else if report.accepted_count > 0 {
        OwnerDecisionImpactFinalStatus::OwnerInputPartiallyApplied
    } else if report.blocked_count > 0 && report.accepted_count == 0 {
        OwnerDecisionImpactFinalStatus::OwnerInputBlocked
    } else {
        OwnerDecisionImpactFinalStatus::OwnerInputDiagnosticOnly
    };
    report.stabilize();
    report
}

fn build_impact_record(
    candidate_panel: &CandidatePanel,
    queue: &super::OwnerReviewQueue,
    input: &OwnerInput,
) -> OwnerDecisionImpactRecord {
    let validation = validate_owner_input(input);
    let before_status = candidate_panel
        .candidates
        .iter()
        .find(|candidate| input.target_id.as_deref() == Some(candidate.candidate_id.as_str()))
        .map(|candidate| format!("{:?}", candidate.status));
    let review_item = queue
        .pending_items
        .iter()
        .chain(queue.reviewed_items.iter())
        .chain(queue.deferred_items.iter())
        .chain(queue.dismissed_items.iter())
        .chain(queue.paper_confirmed_items.iter())
        .chain(queue.blocked_items.iter())
        .chain(queue.expired_items.iter())
        .find(|item| item.candidate_id.as_deref() == input.target_id.as_deref());

    let mut record = OwnerDecisionImpactRecord {
        owner_input_id: input.owner_input_id.clone(),
        target_id: input.target_id.clone(),
        before_status,
        after_status: None,
        impact_kind: OwnerDecisionImpactKind::NoImpact,
        affected_panels: vec!["owner".to_string(), "audit".to_string()],
        reason_codes: validation.reason_codes.clone(),
    };

    if !validation.allowed {
        record.impact_kind = OwnerDecisionImpactKind::BlockedByPolicy;
    } else {
        match input.input_kind {
            OwnerInputKind::CandidateHold => {
                record.impact_kind = OwnerDecisionImpactKind::CandidateHeld;
                record.after_status = Some("Deferred".to_string());
                record.affected_panels.push("candidate".to_string());
            }
            OwnerInputKind::CandidateDismiss => {
                record.impact_kind = OwnerDecisionImpactKind::CandidateDismissed;
                record.after_status = Some("Dismissed".to_string());
                record.affected_panels.push("candidate".to_string());
            }
            OwnerInputKind::CandidateReanalysisRequest => {
                record.impact_kind = OwnerDecisionImpactKind::ReanalysisRequested;
                record.after_status = Some("PendingReview".to_string());
                record
                    .affected_panels
                    .extend(["candidate".to_string(), "chair".to_string()]);
            }
            OwnerInputKind::PaperConfirm => {
                if matches!(
                    review_item.map(|item| item.current_status),
                    Some(super::OwnerReviewItemStatus::BlockedByRiskGovernor)
                ) || matches!(
                    candidate_panel
                        .candidates
                        .iter()
                        .find(|candidate| input.target_id.as_deref()
                            == Some(candidate.candidate_id.as_str()))
                        .map(|candidate| candidate.status),
                    Some(CandidateStatus::RiskBlocked)
                ) {
                    record.impact_kind = OwnerDecisionImpactKind::BlockedByRiskGovernor;
                } else if matches!(
                    review_item.map(|item| item.current_status),
                    Some(super::OwnerReviewItemStatus::PaperConfirmed)
                ) {
                    record.impact_kind = OwnerDecisionImpactKind::PaperConfirmed;
                    record.after_status = Some("PaperConfirmed".to_string());
                    record
                        .affected_panels
                        .extend(["candidate".to_string(), "human_confirm".to_string()]);
                } else {
                    record.impact_kind = OwnerDecisionImpactKind::BlockedByPolicy;
                }
            }
            OwnerInputKind::RiskTightenRequest => {
                record.impact_kind = OwnerDecisionImpactKind::RiskMadeMoreConservative;
                record.affected_panels.push("risk".to_string());
            }
            _ if validation.diagnostic_only => {
                record.impact_kind = OwnerDecisionImpactKind::DiagnosticOnly;
            }
            _ => {
                record.impact_kind = OwnerDecisionImpactKind::NoImpact;
            }
        }
    }

    record.stabilize();
    record
}
