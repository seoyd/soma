use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash,
)]
pub enum CandidateLifecycleStatus {
    #[default]
    Detected,
    WaitingData,
    EvidenceReady,
    UnderAnalysis,
    CommitteeVoting,
    ChairReviewed,
    RiskReview,
    HumanConfirmRequired,
    PaperApproved,
    PaperPositionOpen,
    PaperPositionClosed,
    NoTrade,
    RiskBlocked,
    OwnerHeld,
    OwnerDismissed,
    ReanalysisRequested,
    Expired,
    ResearchOnly,
    DiagnosticOnly,
    Error,
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash,
)]
pub enum CandidateLifecycleEvent {
    #[default]
    Detect,
    WaitForData,
    EvidenceReady,
    StartAnalysis,
    StartVoting,
    ChairReviewComplete,
    BeginRiskReview,
    HumanConfirmRequired,
    PaperApprove,
    OwnerPaperConfirm,
    PaperPositionOpen,
    PaperPositionClosed,
    MarkNoTrade,
    MarkRiskBlocked,
    OwnerHold,
    OwnerDismiss,
    RequestReanalysis,
    Expire,
    ResearchOnlyBoundary,
    DiagnosticOnlyBoundary,
    RaiseError,
    OfficialPaperApprove,
    RealOrder,
    BrokerPosition,
    LiveTrading,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateLifecycleTransition {
    pub from_status: CandidateLifecycleStatus,
    pub event: CandidateLifecycleEvent,
    pub to_status: CandidateLifecycleStatus,
    pub allowed: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateLifecycleStateMachine {
    #[serde(default)]
    pub transitions: Vec<CandidateLifecycleTransition>,
    #[serde(default)]
    pub forbidden_transitions: Vec<CandidateLifecycleTransition>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CandidateLifecycleTransition {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(&serde_json::to_string(self).unwrap_or_else(|_| {
            format!(
                "{:?}:{:?}:{:?}:{}",
                self.from_status, self.event, self.to_status, self.allowed
            )
        }))
    }

    pub fn stabilize(&mut self) {
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

impl CandidateLifecycleStateMachine {
    pub fn transition(
        &self,
        from_status: CandidateLifecycleStatus,
        event: CandidateLifecycleEvent,
    ) -> CandidateLifecycleTransition {
        if let Some(transition) = self
            .transitions
            .iter()
            .find(|transition| transition.from_status == from_status && transition.event == event)
        {
            return transition.clone();
        }
        if let Some(transition) = self
            .forbidden_transitions
            .iter()
            .find(|transition| transition.from_status == from_status && transition.event == event)
        {
            return transition.clone();
        }
        CandidateLifecycleTransition {
            from_status,
            event,
            to_status: from_status,
            allowed: false,
            reason_codes: vec![ReasonCode::DeterministicPath, ReasonCode::CandidateRejected],
        }
    }

    pub fn is_allowed(
        &self,
        from_status: CandidateLifecycleStatus,
        event: CandidateLifecycleEvent,
        to_status: CandidateLifecycleStatus,
    ) -> bool {
        let transition = self.transition(from_status, event);
        transition.allowed && transition.to_status == to_status
    }

    pub fn fingerprint(&self) -> String {
        let mut copy = self.clone();
        copy.stabilize();
        stable_hash_string(&serde_json::to_string(&copy).unwrap_or_default())
    }

    pub fn stabilize(&mut self) {
        self.transitions.sort_by(|left, right| {
            (left.from_status, left.event, left.to_status).cmp(&(
                right.from_status,
                right.event,
                right.to_status,
            ))
        });
        self.forbidden_transitions.sort_by(|left, right| {
            (left.from_status, left.event, left.to_status).cmp(&(
                right.from_status,
                right.event,
                right.to_status,
            ))
        });
        for transition in self
            .transitions
            .iter_mut()
            .chain(self.forbidden_transitions.iter_mut())
        {
            transition.stabilize();
        }
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

impl Default for CandidateLifecycleStateMachine {
    fn default() -> Self {
        let mut machine = Self {
            transitions: allowed_transition_specs()
                .into_iter()
                .map(
                    |(from_status, event, to_status)| CandidateLifecycleTransition {
                        from_status,
                        event,
                        to_status,
                        allowed: true,
                        reason_codes: vec![ReasonCode::DeterministicPath],
                    },
                )
                .collect(),
            forbidden_transitions: forbidden_transition_specs()
                .into_iter()
                .map(
                    |(from_status, event, to_status, reason_codes)| CandidateLifecycleTransition {
                        from_status,
                        event,
                        to_status,
                        allowed: false,
                        reason_codes,
                    },
                )
                .collect(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        };
        machine.stabilize();
        machine
    }
}

fn allowed_transition_specs() -> Vec<(
    CandidateLifecycleStatus,
    CandidateLifecycleEvent,
    CandidateLifecycleStatus,
)> {
    use CandidateLifecycleEvent as Event;
    use CandidateLifecycleStatus as Status;

    vec![
        (Status::Detected, Event::WaitForData, Status::WaitingData),
        (
            Status::Detected,
            Event::EvidenceReady,
            Status::EvidenceReady,
        ),
        (
            Status::WaitingData,
            Event::EvidenceReady,
            Status::EvidenceReady,
        ),
        (
            Status::EvidenceReady,
            Event::StartAnalysis,
            Status::UnderAnalysis,
        ),
        (
            Status::UnderAnalysis,
            Event::StartVoting,
            Status::CommitteeVoting,
        ),
        (
            Status::CommitteeVoting,
            Event::ChairReviewComplete,
            Status::ChairReviewed,
        ),
        (
            Status::ChairReviewed,
            Event::BeginRiskReview,
            Status::RiskReview,
        ),
        (
            Status::RiskReview,
            Event::HumanConfirmRequired,
            Status::HumanConfirmRequired,
        ),
        (
            Status::RiskReview,
            Event::PaperApprove,
            Status::PaperApproved,
        ),
        (
            Status::HumanConfirmRequired,
            Event::OwnerPaperConfirm,
            Status::PaperApproved,
        ),
        (
            Status::PaperApproved,
            Event::PaperPositionOpen,
            Status::PaperPositionOpen,
        ),
        (
            Status::PaperPositionOpen,
            Event::PaperPositionClosed,
            Status::PaperPositionClosed,
        ),
        (Status::RiskReview, Event::MarkNoTrade, Status::NoTrade),
        (
            Status::RiskReview,
            Event::MarkRiskBlocked,
            Status::RiskBlocked,
        ),
        (
            Status::HumanConfirmRequired,
            Event::OwnerHold,
            Status::OwnerHeld,
        ),
        (
            Status::HumanConfirmRequired,
            Event::OwnerDismiss,
            Status::OwnerDismissed,
        ),
        (
            Status::HumanConfirmRequired,
            Event::RequestReanalysis,
            Status::ReanalysisRequested,
        ),
        (
            Status::OwnerHeld,
            Event::RequestReanalysis,
            Status::ReanalysisRequested,
        ),
        (
            Status::ReanalysisRequested,
            Event::StartAnalysis,
            Status::UnderAnalysis,
        ),
        (
            Status::Detected,
            Event::ResearchOnlyBoundary,
            Status::ResearchOnly,
        ),
        (
            Status::Detected,
            Event::DiagnosticOnlyBoundary,
            Status::DiagnosticOnly,
        ),
        (
            Status::EvidenceReady,
            Event::ResearchOnlyBoundary,
            Status::ResearchOnly,
        ),
        (
            Status::EvidenceReady,
            Event::DiagnosticOnlyBoundary,
            Status::DiagnosticOnly,
        ),
        (Status::Detected, Event::Expire, Status::Expired),
        (Status::EvidenceReady, Event::Expire, Status::Expired),
        (
            Status::PaperPositionOpen,
            Event::Expire,
            Status::PaperPositionClosed,
        ),
        (Status::Detected, Event::RaiseError, Status::Error),
        (Status::EvidenceReady, Event::RaiseError, Status::Error),
        (Status::UnderAnalysis, Event::RaiseError, Status::Error),
        (Status::CommitteeVoting, Event::RaiseError, Status::Error),
        (Status::ChairReviewed, Event::RaiseError, Status::Error),
        (Status::RiskReview, Event::RaiseError, Status::Error),
        (
            Status::HumanConfirmRequired,
            Event::RaiseError,
            Status::Error,
        ),
        (Status::PaperApproved, Event::RaiseError, Status::Error),
        (Status::PaperPositionOpen, Event::RaiseError, Status::Error),
    ]
}

fn forbidden_transition_specs() -> Vec<(
    CandidateLifecycleStatus,
    CandidateLifecycleEvent,
    CandidateLifecycleStatus,
    Vec<ReasonCode>,
)> {
    use CandidateLifecycleEvent as Event;
    use CandidateLifecycleStatus as Status;

    vec![
        (
            Status::RiskBlocked,
            Event::OfficialPaperApprove,
            Status::PaperApproved,
            vec![
                ReasonCode::RiskDenied,
                ReasonCode::OwnerCannotBypassRiskGovernor,
            ],
        ),
        (
            Status::NoTrade,
            Event::OfficialPaperApprove,
            Status::PaperApproved,
            vec![ReasonCode::NoTradeDefault],
        ),
        (
            Status::ResearchOnly,
            Event::OfficialPaperApprove,
            Status::PaperApproved,
            vec![ReasonCode::ResearchOnlyOverride],
        ),
        (
            Status::DiagnosticOnly,
            Event::OfficialPaperApprove,
            Status::PaperApproved,
            vec![ReasonCode::ResearchOnlyOverride],
        ),
        (
            Status::Detected,
            Event::RealOrder,
            Status::Error,
            vec![ReasonCode::PaperExecutionOnly],
        ),
        (
            Status::PaperApproved,
            Event::RealOrder,
            Status::Error,
            vec![ReasonCode::PaperExecutionOnly],
        ),
        (
            Status::PaperPositionOpen,
            Event::BrokerPosition,
            Status::Error,
            vec![ReasonCode::PaperExecutionOnly],
        ),
        (
            Status::PaperPositionOpen,
            Event::LiveTrading,
            Status::Error,
            vec![ReasonCode::LiveModeDisabled],
        ),
        (
            Status::PaperApproved,
            Event::LiveTrading,
            Status::Error,
            vec![ReasonCode::LiveModeDisabled],
        ),
        (
            Status::RiskBlocked,
            Event::LiveTrading,
            Status::Error,
            vec![ReasonCode::LiveModeDisabled],
        ),
    ]
}
