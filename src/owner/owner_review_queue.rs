use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};
use crate::ui::{CandidatePanel, CandidateStatus, CandidateView};

use super::{
    HumanConfirmProtocolConfig, HumanConfirmState, OwnerInput, OwnerInputKind,
    evaluate_human_confirm_transition,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerReviewItemStatus {
    #[default]
    PendingReview,
    Reviewed,
    Deferred,
    Dismissed,
    PaperConfirmed,
    BlockedByRiskGovernor,
    Expired,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllowedOwnerAction {
    #[default]
    View,
    AddNote,
    Hold,
    Dismiss,
    RequestReanalysis,
    MarkReviewed,
    PaperConfirm,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForbiddenOwnerAction {
    #[default]
    ExecuteOrder,
    PlaceTrade,
    OverrideRisk,
    EnableLiveTrading,
    ModifyAccount,
    AccessBalance,
    QueryHoldings,
    LoosenHardVeto,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerReviewItem {
    pub review_id: String,
    #[serde(default)]
    pub candidate_id: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub market: Option<String>,
    #[serde(default)]
    pub candidate_status: Option<String>,
    #[serde(default)]
    pub chair_decision: Option<String>,
    #[serde(default)]
    pub risk_decision: Option<String>,
    #[serde(default)]
    pub evidence_status: Option<String>,
    #[serde(default)]
    pub owner_inputs: Vec<OwnerInput>,
    pub current_status: OwnerReviewItemStatus,
    #[serde(default)]
    pub allowed_owner_actions: Vec<AllowedOwnerAction>,
    #[serde(default)]
    pub forbidden_owner_actions: Vec<ForbiddenOwnerAction>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerReviewQueue {
    pub queue_id: String,
    #[serde(default)]
    pub pending_items: Vec<OwnerReviewItem>,
    #[serde(default)]
    pub reviewed_items: Vec<OwnerReviewItem>,
    #[serde(default)]
    pub deferred_items: Vec<OwnerReviewItem>,
    #[serde(default)]
    pub dismissed_items: Vec<OwnerReviewItem>,
    #[serde(default)]
    pub paper_confirmed_items: Vec<OwnerReviewItem>,
    #[serde(default)]
    pub blocked_items: Vec<OwnerReviewItem>,
    #[serde(default)]
    pub expired_items: Vec<OwnerReviewItem>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl OwnerReviewItem {
    pub fn stabilize(&mut self) {
        self.owner_inputs
            .sort_by(|left, right| left.owner_input_id.cmp(&right.owner_input_id));
        for input in &mut self.owner_inputs {
            input.stabilize();
        }
        self.allowed_owner_actions
            .sort_by_key(|action| format!("{action:?}"));
        self.allowed_owner_actions.dedup();
        self.forbidden_owner_actions
            .sort_by_key(|action| format!("{action:?}"));
        self.forbidden_owner_actions.dedup();
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }

    pub fn fingerprint(&self) -> String {
        let mut copy = self.clone();
        copy.stabilize();
        stable_hash_string(&serde_json::to_string(&copy).unwrap_or_else(|_| copy.review_id.clone()))
    }
}

impl OwnerReviewQueue {
    pub fn stabilize(&mut self) {
        for items in [
            &mut self.pending_items,
            &mut self.reviewed_items,
            &mut self.deferred_items,
            &mut self.dismissed_items,
            &mut self.paper_confirmed_items,
            &mut self.blocked_items,
            &mut self.expired_items,
        ] {
            items.sort_by(|left, right| left.review_id.cmp(&right.review_id));
            for item in items.iter_mut() {
                item.stabilize();
            }
        }
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }

    pub fn fingerprint(&self) -> String {
        let mut copy = self.clone();
        copy.stabilize();
        stable_hash_string(&serde_json::to_string(&copy).unwrap_or_else(|_| copy.queue_id.clone()))
    }

    pub fn to_text(&self) -> String {
        [
            "research_only_warning=owner review queue is audited local review state only"
                .to_string(),
            "paper_only_warning=paper confirm is visible only when paper-only policy allows it"
                .to_string(),
            format!("queue_id={}", self.queue_id),
            format!("pending_items={}", self.pending_items.len()),
            format!("reviewed_items={}", self.reviewed_items.len()),
            format!("deferred_items={}", self.deferred_items.len()),
            format!("dismissed_items={}", self.dismissed_items.len()),
            format!("paper_confirmed_items={}", self.paper_confirmed_items.len()),
            format!("blocked_items={}", self.blocked_items.len()),
            format!("expired_items={}", self.expired_items.len()),
            format!("fingerprint={}", self.fingerprint()),
        ]
        .join("\n")
    }
}

pub fn build_owner_review_queue(
    queue_id: &str,
    candidate_panel: &CandidatePanel,
    owner_inputs: &[OwnerInput],
    protocol: &HumanConfirmProtocolConfig,
) -> OwnerReviewQueue {
    let mut queue = OwnerReviewQueue {
        queue_id: queue_id.to_string(),
        pending_items: Vec::new(),
        reviewed_items: Vec::new(),
        deferred_items: Vec::new(),
        dismissed_items: Vec::new(),
        paper_confirmed_items: Vec::new(),
        blocked_items: Vec::new(),
        expired_items: Vec::new(),
        reason_codes: vec![ReasonCode::OwnerReviewQueueBuilt],
    };

    for candidate in &candidate_panel.candidates {
        let mut item = build_review_item(candidate, owner_inputs, protocol);
        item.stabilize();
        match item.current_status {
            OwnerReviewItemStatus::PendingReview => queue.pending_items.push(item),
            OwnerReviewItemStatus::Reviewed => queue.reviewed_items.push(item),
            OwnerReviewItemStatus::Deferred => queue.deferred_items.push(item),
            OwnerReviewItemStatus::Dismissed => queue.dismissed_items.push(item),
            OwnerReviewItemStatus::PaperConfirmed => queue.paper_confirmed_items.push(item),
            OwnerReviewItemStatus::BlockedByRiskGovernor => queue.blocked_items.push(item),
            OwnerReviewItemStatus::Expired => queue.expired_items.push(item),
            OwnerReviewItemStatus::DiagnosticOnly => queue.deferred_items.push(item),
        }
    }

    queue.stabilize();
    queue
}

fn build_review_item(
    candidate: &CandidateView,
    owner_inputs: &[OwnerInput],
    protocol: &HumanConfirmProtocolConfig,
) -> OwnerReviewItem {
    let relevant_inputs = owner_inputs
        .iter()
        .filter(|input| {
            input.target_id.as_deref() == Some(candidate.candidate_id.as_str())
                || input.symbol.as_deref() == Some(candidate.symbol.as_str())
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut allowed_owner_actions = vec![AllowedOwnerAction::View, AllowedOwnerAction::AddNote];
    if protocol.allow_hold {
        allowed_owner_actions.push(AllowedOwnerAction::Hold);
    }
    if protocol.allow_dismiss {
        allowed_owner_actions.push(AllowedOwnerAction::Dismiss);
    }
    if protocol.allow_reanalysis_request {
        allowed_owner_actions.push(AllowedOwnerAction::RequestReanalysis);
    }
    if protocol.allow_mark_reviewed {
        allowed_owner_actions.push(AllowedOwnerAction::MarkReviewed);
    }

    let mut current_status = base_review_status(candidate);
    let mut reason_codes = candidate.reason_codes.clone();
    reason_codes.push(ReasonCode::OwnerReviewQueueBuilt);
    let forbidden_owner_actions = vec![
        ForbiddenOwnerAction::ExecuteOrder,
        ForbiddenOwnerAction::PlaceTrade,
        ForbiddenOwnerAction::OverrideRisk,
        ForbiddenOwnerAction::EnableLiveTrading,
        ForbiddenOwnerAction::ModifyAccount,
        ForbiddenOwnerAction::AccessBalance,
        ForbiddenOwnerAction::QueryHoldings,
        ForbiddenOwnerAction::LoosenHardVeto,
    ];

    let transition_state = transition_state_for_candidate(candidate);
    let paper_confirm_transition =
        evaluate_human_confirm_transition(protocol, transition_state, OwnerInputKind::PaperConfirm);
    if paper_confirm_transition.allowed {
        allowed_owner_actions.push(AllowedOwnerAction::PaperConfirm);
        reason_codes.push(ReasonCode::OwnerPaperConfirmAllowed);
    } else {
        reason_codes.extend(paper_confirm_transition.reason_codes.clone());
    }

    if relevant_inputs
        .iter()
        .any(|input| matches!(input.input_kind, OwnerInputKind::CandidateDismiss))
    {
        current_status = OwnerReviewItemStatus::Dismissed;
    } else if relevant_inputs
        .iter()
        .any(|input| matches!(input.input_kind, OwnerInputKind::CandidateHold))
    {
        current_status = OwnerReviewItemStatus::Deferred;
    } else if relevant_inputs
        .iter()
        .any(|input| matches!(input.input_kind, OwnerInputKind::MarkReviewed))
    {
        current_status = OwnerReviewItemStatus::Reviewed;
    } else if paper_confirm_transition.allowed
        && relevant_inputs
            .iter()
            .any(|input| matches!(input.input_kind, OwnerInputKind::PaperConfirm))
    {
        current_status = OwnerReviewItemStatus::PaperConfirmed;
    }

    if relevant_inputs
        .iter()
        .any(|input| matches!(input.input_kind, OwnerInputKind::CandidateReanalysisRequest))
    {
        reason_codes.push(ReasonCode::OwnerReanalysisRequested);
    }

    OwnerReviewItem {
        review_id: format!("review-{}", candidate.candidate_id),
        candidate_id: Some(candidate.candidate_id.clone()),
        symbol: Some(candidate.symbol.clone()),
        market: Some(candidate.market.clone()),
        candidate_status: Some(format!("{:?}", candidate.status)),
        chair_decision: Some(candidate.chair_summary.clone()),
        risk_decision: Some(candidate.risk_summary.clone()),
        evidence_status: Some(candidate.source_kind.clone()),
        owner_inputs: relevant_inputs,
        current_status,
        allowed_owner_actions,
        forbidden_owner_actions,
        reason_codes,
    }
}

fn base_review_status(candidate: &CandidateView) -> OwnerReviewItemStatus {
    if matches!(candidate.status, CandidateStatus::RiskBlocked) {
        OwnerReviewItemStatus::BlockedByRiskGovernor
    } else if matches!(candidate.status, CandidateStatus::Expired) {
        OwnerReviewItemStatus::Expired
    } else if candidate.source_kind.eq_ignore_ascii_case("researchonly")
        || candidate.source_kind.eq_ignore_ascii_case("fixtureonly")
        || matches!(candidate.status, CandidateStatus::DiagnosticOnly)
    {
        OwnerReviewItemStatus::DiagnosticOnly
    } else {
        OwnerReviewItemStatus::PendingReview
    }
}

fn transition_state_for_candidate(candidate: &CandidateView) -> HumanConfirmState {
    if matches!(candidate.status, CandidateStatus::RiskBlocked) {
        HumanConfirmState::RiskBlocked
    } else if matches!(candidate.status, CandidateStatus::NoTrade) {
        HumanConfirmState::NoTrade
    } else if candidate.source_kind.eq_ignore_ascii_case("researchonly") {
        HumanConfirmState::ResearchOnly
    } else if candidate.source_kind.eq_ignore_ascii_case("fixtureonly")
        || matches!(candidate.status, CandidateStatus::DiagnosticOnly)
    {
        HumanConfirmState::DiagnosticOnly
    } else if matches!(candidate.status, CandidateStatus::HumanConfirmRequired) {
        HumanConfirmState::HumanConfirmRequired
    } else {
        HumanConfirmState::PendingReview
    }
}
