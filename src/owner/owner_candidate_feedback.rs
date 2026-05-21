use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_ordered_strings, stable_reason_codes};

use super::{OwnerInput, OwnerInputKind, OwnerPolicyValidationResult};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerCandidateFeedbackKind {
    #[default]
    Note,
    BullishConcern,
    BearishConcern,
    RiskConcern,
    DataConcern,
    ThesisSupport,
    ThesisConflict,
    Hold,
    Dismiss,
    ReanalysisRequest,
    PaperConfirm,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerFeedbackDecisionEffect {
    #[default]
    NoDirectEffect,
    ChairReanalysisRequested,
    RiskMoreConservativeRequested,
    CandidateHeld,
    CandidateDismissed,
    PaperConfirmed,
    DiagnosticOnly,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerCandidateFeedback {
    pub feedback_id: String,
    pub candidate_id: String,
    pub owner_input_id: String,
    pub feedback_kind: OwnerCandidateFeedbackKind,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub structured_tags: Vec<String>,
    pub affects_decision: OwnerFeedbackDecisionEffect,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl OwnerCandidateFeedback {
    pub fn stabilize(&mut self) {
        self.structured_tags = stable_ordered_strings(&self.structured_tags);
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }

    pub fn fingerprint(&self) -> String {
        let mut copy = self.clone();
        copy.stabilize();
        stable_hash_string(
            &serde_json::to_string(&copy).unwrap_or_else(|_| copy.feedback_id.clone()),
        )
    }
}

pub fn build_owner_candidate_feedback(
    input: &OwnerInput,
    validation: &OwnerPolicyValidationResult,
) -> Option<OwnerCandidateFeedback> {
    let candidate_id = input.target_id.clone()?;
    let mut feedback = OwnerCandidateFeedback {
        feedback_id: format!("feedback-{}", input.owner_input_id),
        candidate_id,
        owner_input_id: input.owner_input_id.clone(),
        feedback_kind: OwnerCandidateFeedbackKind::Note,
        text: input.freeform_note.clone(),
        structured_tags: input
            .structured_payload
            .as_ref()
            .map(|payload| {
                payload
                    .iter()
                    .map(|(key, value)| format!("{key}:{value}"))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        affects_decision: OwnerFeedbackDecisionEffect::NoDirectEffect,
        reason_codes: validation.reason_codes.clone(),
    };

    match input.input_kind {
        OwnerInputKind::CandidateNote => {
            feedback.feedback_kind = OwnerCandidateFeedbackKind::Note;
            feedback.affects_decision = OwnerFeedbackDecisionEffect::NoDirectEffect;
        }
        OwnerInputKind::CandidateHold => {
            feedback.feedback_kind = OwnerCandidateFeedbackKind::Hold;
            feedback.affects_decision = OwnerFeedbackDecisionEffect::CandidateHeld;
        }
        OwnerInputKind::CandidateDismiss => {
            feedback.feedback_kind = OwnerCandidateFeedbackKind::Dismiss;
            feedback.affects_decision = OwnerFeedbackDecisionEffect::CandidateDismissed;
        }
        OwnerInputKind::CandidateReanalysisRequest => {
            feedback.feedback_kind = OwnerCandidateFeedbackKind::ReanalysisRequest;
            feedback.affects_decision = OwnerFeedbackDecisionEffect::ChairReanalysisRequested;
        }
        OwnerInputKind::PaperConfirm => {
            feedback.feedback_kind = OwnerCandidateFeedbackKind::PaperConfirm;
            feedback.affects_decision = if validation.allowed && !validation.diagnostic_only {
                OwnerFeedbackDecisionEffect::PaperConfirmed
            } else {
                OwnerFeedbackDecisionEffect::DiagnosticOnly
            };
        }
        OwnerInputKind::RiskTightenRequest => {
            feedback.feedback_kind = OwnerCandidateFeedbackKind::RiskConcern;
            feedback.affects_decision = OwnerFeedbackDecisionEffect::RiskMoreConservativeRequested;
        }
        OwnerInputKind::DataRequest | OwnerInputKind::EvidenceRequest => {
            feedback.feedback_kind = OwnerCandidateFeedbackKind::DataConcern;
            feedback.affects_decision = OwnerFeedbackDecisionEffect::DiagnosticOnly;
        }
        OwnerInputKind::ThesisNote => {
            feedback.feedback_kind = OwnerCandidateFeedbackKind::ThesisSupport;
            feedback.affects_decision = OwnerFeedbackDecisionEffect::NoDirectEffect;
        }
        _ if validation.diagnostic_only => {
            feedback.affects_decision = OwnerFeedbackDecisionEffect::DiagnosticOnly;
        }
        _ => {}
    }

    feedback.stabilize();
    Some(feedback)
}
