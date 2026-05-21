use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::OwnerInputKind;

fn default_true() -> bool {
    true
}

fn default_max_pending_items() -> usize {
    32
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HumanConfirmState {
    PendingReview,
    Reviewed,
    Deferred,
    Dismissed,
    PaperConfirmed,
    RiskBlocked,
    NoTrade,
    HumanConfirmRequired,
    ResearchOnly,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanConfirmProtocolConfig {
    pub protocol_id: String,
    #[serde(default = "default_true")]
    pub allow_paper_confirm: bool,
    #[serde(default = "default_true")]
    pub allow_dismiss: bool,
    #[serde(default = "default_true")]
    pub allow_hold: bool,
    #[serde(default = "default_true")]
    pub allow_reanalysis_request: bool,
    #[serde(default = "default_true")]
    pub allow_mark_reviewed: bool,
    #[serde(default)]
    pub allow_research_only_confirm: bool,
    #[serde(default)]
    pub allow_diagnostic_only_confirm: bool,
    #[serde(default = "default_true")]
    pub require_reason_for_confirm: bool,
    #[serde(default)]
    pub require_reason_for_dismiss: bool,
    #[serde(default = "default_max_pending_items")]
    pub max_pending_items: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for HumanConfirmProtocolConfig {
    fn default() -> Self {
        Self {
            protocol_id: "owner-human-confirm-v1".to_string(),
            allow_paper_confirm: true,
            allow_dismiss: true,
            allow_hold: true,
            allow_reanalysis_request: true,
            allow_mark_reviewed: true,
            allow_research_only_confirm: false,
            allow_diagnostic_only_confirm: false,
            require_reason_for_confirm: true,
            require_reason_for_dismiss: false,
            max_pending_items: default_max_pending_items(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanConfirmTransition {
    pub from_status: HumanConfirmState,
    pub owner_input_kind: OwnerInputKind,
    pub to_status: HumanConfirmState,
    pub allowed: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanConfirmProtocolReport {
    pub protocol_id: String,
    #[serde(default)]
    pub transitions: Vec<HumanConfirmTransition>,
    #[serde(default)]
    pub allowed_actions_summary: Vec<String>,
    #[serde(default)]
    pub forbidden_actions_summary: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl HumanConfirmProtocolConfig {
    pub fn stabilize(&mut self) {
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

impl HumanConfirmTransition {
    pub fn stabilize(&mut self) {
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

impl HumanConfirmProtocolReport {
    pub fn stabilize(&mut self) {
        self.transitions.sort_by(|left, right| {
            format!("{:?}{:?}", left.from_status, left.owner_input_kind).cmp(&format!(
                "{:?}{:?}",
                right.from_status, right.owner_input_kind
            ))
        });
        for transition in &mut self.transitions {
            transition.stabilize();
        }
        self.allowed_actions_summary.sort();
        self.allowed_actions_summary.dedup();
        self.forbidden_actions_summary.sort();
        self.forbidden_actions_summary.dedup();
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }

    pub fn to_text(&self) -> String {
        [
            "research_only_warning=human confirmation is a local audited paper-only protocol".to_string(),
            "paper_only_warning=paper confirmed items never create broker or live execution actions".to_string(),
            format!("protocol_id={}", self.protocol_id),
            format!("transition_count={}", self.transitions.len()),
            format!("allowed_actions={}", self.allowed_actions_summary.join("|")),
            format!("forbidden_actions={}", self.forbidden_actions_summary.join("|")),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ]
        .join("\n")
    }
}

pub fn evaluate_human_confirm_transition(
    config: &HumanConfirmProtocolConfig,
    from_status: HumanConfirmState,
    owner_input_kind: OwnerInputKind,
) -> HumanConfirmTransition {
    let mut transition = HumanConfirmTransition {
        from_status,
        owner_input_kind,
        to_status: from_status,
        allowed: false,
        reason_codes: vec![ReasonCode::HumanConfirmProtocolBuilt],
    };

    match (from_status, owner_input_kind) {
        (HumanConfirmState::PendingReview, OwnerInputKind::MarkReviewed)
            if config.allow_mark_reviewed =>
        {
            transition.allowed = true;
            transition.to_status = HumanConfirmState::Reviewed;
            transition
                .reason_codes
                .push(ReasonCode::OwnerMarkedReviewed);
        }
        (HumanConfirmState::PendingReview, OwnerInputKind::CandidateDismiss)
            if config.allow_dismiss =>
        {
            transition.allowed = true;
            transition.to_status = HumanConfirmState::Dismissed;
            transition
                .reason_codes
                .push(ReasonCode::OwnerCandidateDismissed);
        }
        (HumanConfirmState::PendingReview, OwnerInputKind::CandidateHold) if config.allow_hold => {
            transition.allowed = true;
            transition.to_status = HumanConfirmState::Deferred;
            transition.reason_codes.push(ReasonCode::OwnerCandidateHeld);
        }
        (HumanConfirmState::HumanConfirmRequired, OwnerInputKind::PaperConfirm)
            if config.allow_paper_confirm =>
        {
            transition.allowed = true;
            transition.to_status = HumanConfirmState::PaperConfirmed;
            transition.reason_codes.extend([
                ReasonCode::OwnerPaperConfirmAllowed,
                ReasonCode::OwnerPaperConfirmPaperOnly,
            ]);
        }
        (HumanConfirmState::ResearchOnly, OwnerInputKind::PaperConfirm)
            if config.allow_paper_confirm && config.allow_research_only_confirm =>
        {
            transition.allowed = true;
            transition.to_status = HumanConfirmState::PaperConfirmed;
            transition.reason_codes.extend([
                ReasonCode::OwnerPaperConfirmAllowed,
                ReasonCode::OwnerPaperConfirmPaperOnly,
            ]);
        }
        (HumanConfirmState::DiagnosticOnly, OwnerInputKind::PaperConfirm)
            if config.allow_paper_confirm && config.allow_diagnostic_only_confirm =>
        {
            transition.allowed = true;
            transition.to_status = HumanConfirmState::PaperConfirmed;
            transition.reason_codes.extend([
                ReasonCode::OwnerPaperConfirmAllowed,
                ReasonCode::OwnerPaperConfirmPaperOnly,
            ]);
        }
        (HumanConfirmState::RiskBlocked, OwnerInputKind::PaperConfirm) => {
            transition.reason_codes.extend([
                ReasonCode::OwnerPaperConfirmBlocked,
                ReasonCode::RiskDenied,
                ReasonCode::OwnerCannotBypassRiskGovernor,
            ]);
        }
        (HumanConfirmState::NoTrade, OwnerInputKind::PaperConfirm) => {
            transition.reason_codes.extend([
                ReasonCode::OwnerPaperConfirmBlocked,
                ReasonCode::NoTradeDefault,
            ]);
        }
        (HumanConfirmState::ResearchOnly, OwnerInputKind::PaperConfirm) => {
            transition.reason_codes.extend([
                ReasonCode::OwnerResearchOnlyConfirmBlocked,
                ReasonCode::OwnerPaperConfirmBlocked,
            ]);
        }
        (HumanConfirmState::DiagnosticOnly, OwnerInputKind::PaperConfirm) => {
            transition.reason_codes.extend([
                ReasonCode::OwnerDiagnosticOnlyConfirmBlocked,
                ReasonCode::OwnerPaperConfirmBlocked,
            ]);
        }
        _ => {
            transition
                .reason_codes
                .push(ReasonCode::OwnerInputDiagnosticOnly);
        }
    }

    transition.stabilize();
    transition
}

pub fn build_human_confirm_protocol_report(
    config: &HumanConfirmProtocolConfig,
) -> HumanConfirmProtocolReport {
    let transition_specs = [
        (
            HumanConfirmState::PendingReview,
            OwnerInputKind::MarkReviewed,
        ),
        (
            HumanConfirmState::PendingReview,
            OwnerInputKind::CandidateDismiss,
        ),
        (
            HumanConfirmState::PendingReview,
            OwnerInputKind::CandidateHold,
        ),
        (
            HumanConfirmState::HumanConfirmRequired,
            OwnerInputKind::PaperConfirm,
        ),
        (HumanConfirmState::RiskBlocked, OwnerInputKind::PaperConfirm),
        (HumanConfirmState::NoTrade, OwnerInputKind::PaperConfirm),
        (
            HumanConfirmState::ResearchOnly,
            OwnerInputKind::PaperConfirm,
        ),
        (
            HumanConfirmState::DiagnosticOnly,
            OwnerInputKind::PaperConfirm,
        ),
    ];
    let transitions = transition_specs
        .into_iter()
        .map(|(from_status, owner_input_kind)| {
            evaluate_human_confirm_transition(config, from_status, owner_input_kind)
        })
        .collect::<Vec<_>>();
    let mut report = HumanConfirmProtocolReport {
        protocol_id: config.protocol_id.clone(),
        transitions,
        allowed_actions_summary: vec![
            "View".to_string(),
            "AddNote".to_string(),
            "Hold".to_string(),
            "Dismiss".to_string(),
            "RequestReanalysis".to_string(),
            "MarkReviewed".to_string(),
            "PaperConfirm".to_string(),
        ],
        forbidden_actions_summary: vec![
            "ExecuteOrder".to_string(),
            "PlaceTrade".to_string(),
            "OverrideRisk".to_string(),
            "EnableLiveTrading".to_string(),
            "ModifyAccount".to_string(),
            "AccessBalance".to_string(),
            "QueryHoldings".to_string(),
            "LoosenHardVeto".to_string(),
        ],
        reason_codes: vec![ReasonCode::HumanConfirmProtocolBuilt],
    };
    report.stabilize();
    report
}
