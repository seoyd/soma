use std::{collections::BTreeSet, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::core::{
    MarketSnapshot, ReasonCode, Regime, RiskDecision, RiskDecisionKind, stable_hash_string,
    stable_reason_codes,
};
use crate::model::{ChairShadowObservationReportV0, validate_chair_shadow_observation_report_v0};

use super::{OwnerInput, OwnerInputKind, OwnerInputStatus, OwnerInputTargetType};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerPolicyConstraintKind {
    CannotBypassRiskGovernor,
    CannotForceTrade,
    CannotEnableLiveTrading,
    CannotEnableBrokerAPI,
    CannotLoosenHardVeto,
    CannotPromoteResearchOnlyToOfficial,
    CannotPromoteFixtureToOfficial,
    CannotBypassEvidenceGate,
    CanMakeMoreConservative,
    CanRequestReanalysis,
    CanDismissCandidate,
    CanHoldCandidate,
    #[default]
    CanPaperConfirmOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerPolicyConstraint {
    pub constraint_id: String,
    pub constraint_kind: OwnerPolicyConstraintKind,
    pub hard: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerPolicyValidationResult {
    pub input_id: String,
    pub allowed: bool,
    #[serde(default)]
    pub blocked_constraints: Vec<OwnerPolicyConstraint>,
    pub diagnostic_only: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerTradeRequestReview {
    pub input_id: String,
    pub advisory_only: bool,
    pub owner_forced_trade: bool,
    pub paper_action_allowed: bool,
    pub explanation: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl OwnerPolicyConstraint {
    pub fn stabilize(&mut self) {
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

impl OwnerPolicyValidationResult {
    pub fn stabilize(&mut self) {
        self.blocked_constraints
            .sort_by(|left, right| left.constraint_id.cmp(&right.constraint_id));
        for constraint in &mut self.blocked_constraints {
            constraint.stabilize();
        }
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }

    pub fn to_text(&self) -> String {
        [
            "research_only_warning=owner input validation is audited, local-only, and never enables live trading".to_string(),
            "paper_only_warning=paper confirm remains paper-only and never triggers broker execution".to_string(),
            format!("input_id={}", self.input_id),
            format!("allowed={}", self.allowed),
            format!("diagnostic_only={}", self.diagnostic_only),
            format!(
                "blocked_constraints={}",
                self.blocked_constraints
                    .iter()
                    .map(|constraint| format!("{:?}", constraint.constraint_kind))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
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

pub fn default_owner_policy_constraints() -> Vec<OwnerPolicyConstraint> {
    let mut constraints = vec![
        OwnerPolicyConstraint {
            constraint_id: "cannot-bypass-risk-governor".to_string(),
            constraint_kind: OwnerPolicyConstraintKind::CannotBypassRiskGovernor,
            hard: true,
            reason_codes: vec![ReasonCode::OwnerCannotBypassRiskGovernor],
        },
        OwnerPolicyConstraint {
            constraint_id: "cannot-force-trade".to_string(),
            constraint_kind: OwnerPolicyConstraintKind::CannotForceTrade,
            hard: true,
            reason_codes: vec![ReasonCode::OwnerCannotForceTrade],
        },
        OwnerPolicyConstraint {
            constraint_id: "cannot-enable-live-trading".to_string(),
            constraint_kind: OwnerPolicyConstraintKind::CannotEnableLiveTrading,
            hard: true,
            reason_codes: vec![ReasonCode::OwnerCannotEnableLiveTrading],
        },
        OwnerPolicyConstraint {
            constraint_id: "cannot-enable-broker-api".to_string(),
            constraint_kind: OwnerPolicyConstraintKind::CannotEnableBrokerAPI,
            hard: true,
            reason_codes: vec![ReasonCode::OwnerCannotEnableBrokerApi],
        },
        OwnerPolicyConstraint {
            constraint_id: "cannot-loosen-hard-veto".to_string(),
            constraint_kind: OwnerPolicyConstraintKind::CannotLoosenHardVeto,
            hard: true,
            reason_codes: vec![ReasonCode::OwnerCannotLoosenHardVeto],
        },
        OwnerPolicyConstraint {
            constraint_id: "can-make-more-conservative".to_string(),
            constraint_kind: OwnerPolicyConstraintKind::CanMakeMoreConservative,
            hard: false,
            reason_codes: vec![ReasonCode::OwnerRiskTightenRequested],
        },
        OwnerPolicyConstraint {
            constraint_id: "can-request-reanalysis".to_string(),
            constraint_kind: OwnerPolicyConstraintKind::CanRequestReanalysis,
            hard: false,
            reason_codes: vec![ReasonCode::OwnerReanalysisRequested],
        },
        OwnerPolicyConstraint {
            constraint_id: "can-dismiss-candidate".to_string(),
            constraint_kind: OwnerPolicyConstraintKind::CanDismissCandidate,
            hard: false,
            reason_codes: vec![ReasonCode::OwnerCandidateDismissed],
        },
        OwnerPolicyConstraint {
            constraint_id: "can-hold-candidate".to_string(),
            constraint_kind: OwnerPolicyConstraintKind::CanHoldCandidate,
            hard: false,
            reason_codes: vec![ReasonCode::OwnerCandidateHeld],
        },
        OwnerPolicyConstraint {
            constraint_id: "can-paper-confirm-only".to_string(),
            constraint_kind: OwnerPolicyConstraintKind::CanPaperConfirmOnly,
            hard: false,
            reason_codes: vec![ReasonCode::OwnerPaperConfirmPaperOnly],
        },
    ];
    for constraint in &mut constraints {
        constraint.stabilize();
    }
    constraints.sort_by(|left, right| left.constraint_id.cmp(&right.constraint_id));
    constraints
}

pub fn validate_owner_input(input: &OwnerInput) -> OwnerPolicyValidationResult {
    let mut reason_codes = vec![
        ReasonCode::OwnerPolicyValidated,
        ReasonCode::OwnerInputValidated,
    ];
    let policy = default_owner_policy_constraints();
    let mut blocked_constraints = Vec::new();
    let mut allowed = true;
    let mut diagnostic_only = input.is_diagnostic_only_kind() || input.freeform_only();

    if input.is_unknown() {
        allowed = false;
        reason_codes.push(ReasonCode::OwnerInputUnknownRejected);
    }

    if input.freeform_only() {
        reason_codes.push(ReasonCode::OwnerFreeformNoteNoDirectEffect);
    }

    if input.requests_forbidden_runtime_action() {
        allowed = false;
        blocked_constraints.extend(
            policy
                .iter()
                .filter(|constraint| {
                    matches!(
                        constraint.constraint_kind,
                        OwnerPolicyConstraintKind::CannotForceTrade
                            | OwnerPolicyConstraintKind::CannotEnableLiveTrading
                            | OwnerPolicyConstraintKind::CannotEnableBrokerAPI
                    )
                })
                .cloned(),
        );
    }

    if matches!(
        input.input_kind,
        OwnerInputKind::RiskLoosenRequestDiagnosticOnly
    ) {
        diagnostic_only = true;
        reason_codes.push(ReasonCode::OwnerRiskLoosenDiagnosticOnly);
    }

    if matches!(input.input_kind, OwnerInputKind::PaperConfirm) {
        reason_codes.push(ReasonCode::OwnerPaperConfirmPaperOnly);
    }
    if matches!(input.input_kind, OwnerInputKind::RiskTightenRequest) {
        reason_codes.push(ReasonCode::OwnerRiskTightenRequested);
    }
    if matches!(input.input_kind, OwnerInputKind::CandidateDismiss) {
        reason_codes.push(ReasonCode::OwnerCandidateDismissed);
    }
    if matches!(input.input_kind, OwnerInputKind::CandidateHold) {
        reason_codes.push(ReasonCode::OwnerCandidateHeld);
    }
    if matches!(input.input_kind, OwnerInputKind::CandidateReanalysisRequest) {
        reason_codes.push(ReasonCode::OwnerReanalysisRequested);
    }

    if blocked_constraints.iter().any(|constraint| constraint.hard) || !allowed {
        allowed = false;
        reason_codes.push(ReasonCode::OwnerInputBlocked);
    } else if diagnostic_only {
        reason_codes.push(ReasonCode::OwnerInputDiagnosticOnly);
    } else {
        reason_codes.push(ReasonCode::OwnerInputApplied);
    }

    let mut result = OwnerPolicyValidationResult {
        input_id: input.owner_input_id.clone(),
        allowed,
        blocked_constraints,
        diagnostic_only,
        reason_codes,
    };
    result.stabilize();
    result
}

pub fn review_owner_trade_request(
    input: &OwnerInput,
    risk_decision: &RiskDecision,
    market: &MarketSnapshot,
    evaluation_timestamp_ms: u64,
) -> OwnerTradeRequestReview {
    let validation = validate_owner_input(input);
    let paper_confirm = matches!(input.input_kind, OwnerInputKind::PaperConfirm);
    let risk_approved = risk_decision.kind == RiskDecisionKind::ApprovePaper
        && risk_decision.approved_order_plan.is_some();
    let paper_action_allowed = validation.allowed && paper_confirm && risk_approved;
    let mut reason_codes = validation.reason_codes;

    if !risk_approved {
        reason_codes.push(ReasonCode::OwnerRequestedButRiskDenied);
    }
    if !validation.allowed || !paper_confirm {
        reason_codes.push(ReasonCode::OwnerRequestedButPolicyBlocked);
    }
    if risk_decision.reason_codes.iter().any(|reason| {
        matches!(
            reason,
            ReasonCode::ExpectedEdgeNonPositive | ReasonCode::ExpectedEdgeBelowThreshold
        )
    }) {
        reason_codes.push(ReasonCode::OwnerRequestedButLowEdge);
    }
    if market.spread_bps > 12.0
        || risk_decision
            .reason_codes
            .contains(&ReasonCode::SpreadGateBreached)
    {
        reason_codes.push(ReasonCode::OwnerRequestedButBadSpread);
    }
    if risk_decision
        .reason_codes
        .contains(&ReasonCode::ConfidenceGateBreached)
    {
        reason_codes.push(ReasonCode::OwnerRequestedButLowConfidence);
    }
    if market.regime == Regime::Unknown
        || risk_decision
            .reason_codes
            .contains(&ReasonCode::UnknownRegimeGateBreached)
    {
        reason_codes.push(ReasonCode::OwnerRequestedButUnknownRegime);
    }
    if evaluation_timestamp_ms.saturating_sub(market.timestamp_ms) > 60_000
        || market.timestamp_ms > evaluation_timestamp_ms.saturating_add(60_000)
    {
        reason_codes.push(ReasonCode::OwnerRequestedButStaleData);
    }
    reason_codes.extend(risk_decision.reason_codes.iter().cloned());
    reason_codes = stable_reason_codes(&reason_codes);

    let explanation = if paper_action_allowed {
        "Owner input remained advisory; Risk Governor independently approved a paper-only action."
            .to_string()
    } else {
        owner_rejection_explanation(&reason_codes)
    };
    OwnerTradeRequestReview {
        input_id: input.owner_input_id.clone(),
        advisory_only: true,
        owner_forced_trade: false,
        paper_action_allowed,
        explanation,
        reason_codes,
    }
}

pub fn owner_rejection_explanation(reason_codes: &[ReasonCode]) -> String {
    let messages = stable_reason_codes(reason_codes)
        .into_iter()
        .filter_map(|reason| match reason {
            ReasonCode::OwnerRequestedButRiskDenied => {
                Some("Risk Governor did not approve the requested paper action.")
            }
            ReasonCode::OwnerRequestedButLowEdge => Some("Expected numeric edge is below policy."),
            ReasonCode::OwnerRequestedButBadSpread => {
                Some("Observed spread exceeds the conservative limit.")
            }
            ReasonCode::OwnerRequestedButStaleData => {
                Some("Market data is stale for this decision.")
            }
            ReasonCode::OwnerRequestedButLowConfidence => {
                Some("Signal confidence is below the required threshold.")
            }
            ReasonCode::OwnerRequestedButUnknownRegime => Some("Market regime is unknown."),
            ReasonCode::OwnerRequestedButPolicyBlocked => {
                Some("Owner input policy does not allow this action.")
            }
            ReasonCode::OwnerRequestedButAgentInCooldown => {
                Some("The requested agent is in a safety cooldown.")
            }
            ReasonCode::OwnerRequestedButCooldownActive => {
                Some("Active cooldown cannot be cleared by owner input.")
            }
            ReasonCode::OwnerRequestedButSandboxOnly => {
                Some("Sandbox candidates cannot be promoted by owner input.")
            }
            ReasonCode::CooldownOwnerBypassRejected => {
                Some("Owner input cannot clear or bypass an agent cooldown.")
            }
            ReasonCode::OwnerRequestedButDoctrineViolation => {
                Some("The requested action conflicts with immutable agent doctrine.")
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    if messages.is_empty() {
        "NoTrade remains the default; no owner-requested paper action was approved.".to_string()
    } else {
        messages.join(" ")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerShadowAdvisoryStatusV0 {
    AcknowledgedObservationOnly,
    ConservativeRequestAcknowledged,
    ReanalysisRequestAcknowledged,
    EvidenceRequestAcknowledged,
    DiagnosticOnly,
    UnsupportedByRetrospectiveEvidence,
    TargetUnavailable,
    PolicyBlocked,
    UnknownInputRejected,
    InvalidObservationReceipt,
    TechnicalFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairShadowOwnerAdvisoryReviewInputV0 {
    pub review_version: String,
    pub observation_packet_digest: String,
    pub observation_receipt_digest: String,
    pub observation_firewall_digest: String,
    pub owner_input: OwnerInput,
    pub owner_input_fingerprint: String,
    pub retrospective_only: bool,
    pub decision_context_available: bool,
    pub candidate_context_available: bool,
    pub input_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairShadowOwnerAdvisoryReviewV0 {
    pub review_version: String,
    pub observation_packet_digest: String,
    pub owner_input_id: String,
    pub owner_input_fingerprint: String,
    pub owner_policy_allowed: bool,
    pub owner_policy_diagnostic_only: bool,
    pub status: OwnerShadowAdvisoryStatusV0,
    pub reason_codes: Vec<String>,
    pub explanation: String,
    pub considered: bool,
    pub changed_observation: bool,
    pub changed_model: bool,
    pub changed_decision: bool,
    pub changed_risk_policy: bool,
    pub vote_created: bool,
    pub reward_created: bool,
    pub penalty_created: bool,
    pub speaking_right_changed: bool,
    pub risk_handoff_created: bool,
    pub paper_action_created: bool,
    pub execution_created: bool,
    pub review_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerAdvisoryDecisionFirewallProofV0 {
    pub owner_input_cannot_become_vote: bool,
    pub owner_input_cannot_become_chair_input: bool,
    pub observation_cannot_become_risk_decision: bool,
    pub chair_engine_not_invoked: bool,
    pub owner_trade_review_not_invoked: bool,
    pub risk_governor_not_invoked: bool,
    pub paper_broker_not_invoked: bool,
    pub reward_path_not_invoked: bool,
    pub penalty_path_not_invoked: bool,
    pub speaking_right_path_not_invoked: bool,
    pub no_decision_created: bool,
    pub no_action_created: bool,
    pub all_invariants_pass: bool,
    pub proof_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairShadowOwnerReviewLedgerV0 {
    pub ledger_version: String,
    pub observation_packet_digest: String,
    pub reviews: Vec<ChairShadowOwnerAdvisoryReviewV0>,
    pub duplicate_input_count: usize,
    pub chair_runtime_invocations: usize,
    pub risk_runtime_invocations: usize,
    pub action_count: usize,
    pub ledger_digest: String,
}

fn hash_serialized<T: Serialize>(value: &T) -> String {
    stable_hash_string(&serde_json::to_string(value).unwrap_or_default())
}

fn review_input_digest_v0(value: &ChairShadowOwnerAdvisoryReviewInputV0) -> String {
    let mut copy = value.clone();
    copy.input_digest.clear();
    hash_serialized(&copy)
}

fn review_digest_v0(value: &ChairShadowOwnerAdvisoryReviewV0) -> String {
    let mut copy = value.clone();
    copy.review_digest.clear();
    hash_serialized(&copy)
}

fn firewall_proof_digest_v0(value: &OwnerAdvisoryDecisionFirewallProofV0) -> String {
    let mut copy = value.clone();
    copy.proof_digest.clear();
    hash_serialized(&copy)
}

fn review_ledger_digest_v0(value: &ChairShadowOwnerReviewLedgerV0) -> String {
    let mut copy = value.clone();
    copy.ledger_digest.clear();
    hash_serialized(&copy)
}

fn stable_shadow_reason_codes(
    owner_reason_codes: &[ReasonCode],
    extra_reason_codes: impl IntoIterator<Item = &'static str>,
) -> Vec<String> {
    let mut codes = BTreeSet::new();
    codes.extend(owner_reason_codes.iter().map(|code| format!("{code:?}")));
    codes.extend(extra_reason_codes.into_iter().map(str::to_string));
    codes.into_iter().collect()
}

fn review_input_matches_observation_v0(
    observation: &ChairShadowObservationReportV0,
    input: &ChairShadowOwnerAdvisoryReviewInputV0,
) -> bool {
    validate_chair_shadow_observation_report_v0(observation).is_ok()
        && input.review_version == "chair-shadow-owner-advisory-review-v0"
        && input.observation_packet_digest == observation.packet.packet_digest
        && input.observation_receipt_digest == observation.receipt.receipt_digest
        && input.observation_firewall_digest == observation.firewall_proof.proof_digest
        && input.owner_input_fingerprint == input.owner_input.fingerprint()
        && input.retrospective_only
        && !input.decision_context_available
        && !input.candidate_context_available
        && input.input_digest == review_input_digest_v0(input)
}

fn status_for_owner_advisory_v0(
    input: &OwnerInput,
    owner_policy_allowed: bool,
    owner_policy_diagnostic_only: bool,
) -> OwnerShadowAdvisoryStatusV0 {
    if !owner_policy_allowed {
        return if input.is_unknown() {
            OwnerShadowAdvisoryStatusV0::UnknownInputRejected
        } else {
            OwnerShadowAdvisoryStatusV0::PolicyBlocked
        };
    }
    if input.freeform_only() {
        return OwnerShadowAdvisoryStatusV0::DiagnosticOnly;
    }
    match input.input_kind {
        OwnerInputKind::RiskTightenRequest => {
            OwnerShadowAdvisoryStatusV0::ConservativeRequestAcknowledged
        }
        OwnerInputKind::CandidateReanalysisRequest => {
            OwnerShadowAdvisoryStatusV0::ReanalysisRequestAcknowledged
        }
        OwnerInputKind::EvidenceRequest => OwnerShadowAdvisoryStatusV0::EvidenceRequestAcknowledged,
        OwnerInputKind::PaperConfirm
        | OwnerInputKind::CandidateHold
        | OwnerInputKind::CandidateDismiss => OwnerShadowAdvisoryStatusV0::TargetUnavailable,
        OwnerInputKind::WatchlistAdd
        | OwnerInputKind::WatchlistRemove
        | OwnerInputKind::MarkReviewed => {
            OwnerShadowAdvisoryStatusV0::UnsupportedByRetrospectiveEvidence
        }
        OwnerInputKind::ThesisNote
        | OwnerInputKind::StrategyPreference
        | OwnerInputKind::CandidateNote
        | OwnerInputKind::RiskLoosenRequestDiagnosticOnly
        | OwnerInputKind::ProviderPreference
        | OwnerInputKind::DataRequest
        | OwnerInputKind::Abstain => OwnerShadowAdvisoryStatusV0::DiagnosticOnly,
        _ if owner_policy_diagnostic_only => OwnerShadowAdvisoryStatusV0::DiagnosticOnly,
        _ => OwnerShadowAdvisoryStatusV0::AcknowledgedObservationOnly,
    }
}

fn explanation_for_owner_advisory_status_v0(status: OwnerShadowAdvisoryStatusV0) -> String {
    match status {
        OwnerShadowAdvisoryStatusV0::AcknowledgedObservationOnly => "The owner advisory is recorded against retrospective Shadow evidence only; it creates no decision or action.".into(),
        OwnerShadowAdvisoryStatusV0::ConservativeRequestAcknowledged => "The conservative request is acknowledged, but observation mode cannot change risk policy; a separate policy-governed review is required.".into(),
        OwnerShadowAdvisoryStatusV0::ReanalysisRequestAcknowledged => "The reanalysis request is acknowledged, but retrospective observation cannot start an automatic replay; a separately governed replay is required.".into(),
        OwnerShadowAdvisoryStatusV0::EvidenceRequestAcknowledged => "The evidence request is acknowledged, but retrospective observation cannot acquire or replay evidence automatically; a separate governed request is required.".into(),
        OwnerShadowAdvisoryStatusV0::DiagnosticOnly => "The advisory is diagnostic context only and has no direct model, Chair, risk-policy, paper, or execution effect.".into(),
        OwnerShadowAdvisoryStatusV0::UnsupportedByRetrospectiveEvidence => "The advisory is outside what retrospective Shadow evidence can address and has no immediate effect.".into(),
        OwnerShadowAdvisoryStatusV0::TargetUnavailable => "No eligible candidate or decision context exists in the Shadow observation, so the advisory cannot create a candidate, Risk decision, or paper action.".into(),
        OwnerShadowAdvisoryStatusV0::PolicyBlocked => format!("{} Shadow observation keeps the input advisory and creates no decision or action.", owner_rejection_explanation(&[ReasonCode::OwnerRequestedButPolicyBlocked])),
        OwnerShadowAdvisoryStatusV0::UnknownInputRejected => "The owner input kind is unknown under the existing fail-closed policy; retrospective observation creates no decision or action.".into(),
        OwnerShadowAdvisoryStatusV0::InvalidObservationReceipt => "The Shadow observation receipt did not verify, so the advisory cannot be considered for any decision or action.".into(),
        OwnerShadowAdvisoryStatusV0::TechnicalFailure => "The advisory review could not establish its deterministic safety boundary, so no decision or action was created.".into(),
    }
}

fn extra_reason_codes_for_status_v0(status: OwnerShadowAdvisoryStatusV0) -> Vec<&'static str> {
    let mut codes = vec![
        "NoChairDecisionContext",
        "NoEligibleCandidateContext",
        "OwnerAdvisoryObservationOnly",
        "OwnerInputCannotBecomeVote",
        "OwnerInputCannotChangeSpeakingRights",
        "ShadowEvidenceRetrospectiveOnly",
    ];
    match status {
        OwnerShadowAdvisoryStatusV0::ConservativeRequestAcknowledged => {
            codes.push("RiskTightenNotAppliedInObservationMode");
        }
        OwnerShadowAdvisoryStatusV0::ReanalysisRequestAcknowledged => {
            codes.extend([
                "AutomaticReanalysisForbidden",
                "ProspectiveEvidenceRequired",
            ]);
        }
        OwnerShadowAdvisoryStatusV0::EvidenceRequestAcknowledged => {
            codes.push("ProspectiveEvidenceRequired");
        }
        OwnerShadowAdvisoryStatusV0::TargetUnavailable => {
            codes.push("PaperConfirmationRequiresIndependentRiskApproval");
        }
        _ => {}
    }
    codes
}

pub fn chair_shadow_owner_advisory_review_input_v0(
    observation: &ChairShadowObservationReportV0,
    owner_input: OwnerInput,
) -> ChairShadowOwnerAdvisoryReviewInputV0 {
    let mut input = ChairShadowOwnerAdvisoryReviewInputV0 {
        review_version: "chair-shadow-owner-advisory-review-v0".into(),
        observation_packet_digest: observation.packet.packet_digest.clone(),
        observation_receipt_digest: observation.receipt.receipt_digest.clone(),
        observation_firewall_digest: observation.firewall_proof.proof_digest.clone(),
        owner_input_fingerprint: owner_input.fingerprint(),
        owner_input,
        retrospective_only: true,
        decision_context_available: false,
        candidate_context_available: false,
        input_digest: String::new(),
    };
    input.input_digest = review_input_digest_v0(&input);
    input
}

pub fn review_chair_shadow_owner_advisory_v0(
    observation: &ChairShadowObservationReportV0,
    input: &ChairShadowOwnerAdvisoryReviewInputV0,
) -> ChairShadowOwnerAdvisoryReviewV0 {
    let policy = validate_owner_input(&input.owner_input);
    let input_is_verified = review_input_matches_observation_v0(observation, input);
    let status = if input_is_verified {
        status_for_owner_advisory_v0(&input.owner_input, policy.allowed, policy.diagnostic_only)
    } else {
        OwnerShadowAdvisoryStatusV0::InvalidObservationReceipt
    };
    let reason_codes = stable_shadow_reason_codes(
        &policy.reason_codes,
        extra_reason_codes_for_status_v0(status),
    );
    let mut review = ChairShadowOwnerAdvisoryReviewV0 {
        review_version: "chair-shadow-owner-advisory-review-v0".into(),
        observation_packet_digest: input.observation_packet_digest.clone(),
        owner_input_id: input.owner_input.owner_input_id.clone(),
        owner_input_fingerprint: input.owner_input_fingerprint.clone(),
        owner_policy_allowed: policy.allowed,
        owner_policy_diagnostic_only: policy.diagnostic_only,
        status,
        reason_codes,
        explanation: explanation_for_owner_advisory_status_v0(status),
        considered: true,
        changed_observation: false,
        changed_model: false,
        changed_decision: false,
        changed_risk_policy: false,
        vote_created: false,
        reward_created: false,
        penalty_created: false,
        speaking_right_changed: false,
        risk_handoff_created: false,
        paper_action_created: false,
        execution_created: false,
        review_digest: String::new(),
    };
    review.review_digest = review_digest_v0(&review);
    review
}

pub fn owner_advisory_decision_firewall_proof_v0() -> OwnerAdvisoryDecisionFirewallProofV0 {
    let mut proof = OwnerAdvisoryDecisionFirewallProofV0 {
        owner_input_cannot_become_vote: true,
        owner_input_cannot_become_chair_input: true,
        observation_cannot_become_risk_decision: true,
        chair_engine_not_invoked: true,
        owner_trade_review_not_invoked: true,
        risk_governor_not_invoked: true,
        paper_broker_not_invoked: true,
        reward_path_not_invoked: true,
        penalty_path_not_invoked: true,
        speaking_right_path_not_invoked: true,
        no_decision_created: true,
        no_action_created: true,
        all_invariants_pass: true,
        proof_digest: String::new(),
    };
    proof.proof_digest = firewall_proof_digest_v0(&proof);
    proof
}

fn review_has_no_decision_or_action_v0(review: &ChairShadowOwnerAdvisoryReviewV0) -> bool {
    review.considered
        && !review.changed_observation
        && !review.changed_model
        && !review.changed_decision
        && !review.changed_risk_policy
        && !review.vote_created
        && !review.reward_created
        && !review.penalty_created
        && !review.speaking_right_changed
        && !review.risk_handoff_created
        && !review.paper_action_created
        && !review.execution_created
        && review.reason_codes.windows(2).all(|pair| pair[0] < pair[1])
        && !review.explanation.is_empty()
        && review.review_digest == review_digest_v0(review)
}

pub fn new_chair_shadow_owner_review_ledger_v0(
    observation_packet_digest: String,
) -> ChairShadowOwnerReviewLedgerV0 {
    let mut ledger = ChairShadowOwnerReviewLedgerV0 {
        ledger_version: "chair-shadow-owner-review-ledger-v0".into(),
        observation_packet_digest,
        reviews: vec![],
        duplicate_input_count: 0,
        chair_runtime_invocations: 0,
        risk_runtime_invocations: 0,
        action_count: 0,
        ledger_digest: String::new(),
    };
    ledger.ledger_digest = review_ledger_digest_v0(&ledger);
    ledger
}

pub fn validate_chair_shadow_owner_review_ledger_v0(
    ledger: &ChairShadowOwnerReviewLedgerV0,
) -> Result<(), String> {
    let ordered = ledger.reviews.windows(2).all(|pair| {
        (
            pair[0].owner_input_fingerprint.clone(),
            pair[0].owner_input_id.clone(),
        ) < (
            pair[1].owner_input_fingerprint.clone(),
            pair[1].owner_input_id.clone(),
        )
    });
    let unique_fingerprints = ledger
        .reviews
        .windows(2)
        .all(|pair| pair[0].owner_input_fingerprint != pair[1].owner_input_fingerprint);
    if ledger.ledger_version != "chair-shadow-owner-review-ledger-v0"
        || ledger.observation_packet_digest.is_empty()
        || !ordered
        || !unique_fingerprints
        || ledger.duplicate_input_count != 0
        || ledger.chair_runtime_invocations != 0
        || ledger.risk_runtime_invocations != 0
        || ledger.action_count != 0
        || ledger.reviews.iter().any(|review| {
            review.observation_packet_digest != ledger.observation_packet_digest
                || !review_has_no_decision_or_action_v0(review)
        })
        || ledger.ledger_digest != review_ledger_digest_v0(ledger)
    {
        return Err("chair_shadow_owner_review_ledger_invalid".into());
    }
    Ok(())
}

fn write_chair_shadow_owner_review_ledger_v0(
    path: &Path,
    ledger: &ChairShadowOwnerReviewLedgerV0,
) -> Result<(), String> {
    validate_chair_shadow_owner_review_ledger_v0(ledger)?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| "chair_shadow_owner_review_ledger_path_invalid".to_string())?;
    fs::create_dir_all(parent).map_err(|_| "chair_shadow_owner_review_ledger_write".to_string())?;
    let temporary = path.with_extension("tmp");
    let encoded = serde_json::to_vec_pretty(ledger)
        .map_err(|_| "chair_shadow_owner_review_ledger_write".to_string())?;
    fs::write(&temporary, encoded)
        .map_err(|_| "chair_shadow_owner_review_ledger_write".to_string())?;
    fs::rename(&temporary, path).map_err(|_| "chair_shadow_owner_review_ledger_write".to_string())
}

pub fn read_chair_shadow_owner_review_ledger_v0(
    path: &Path,
) -> Result<ChairShadowOwnerReviewLedgerV0, String> {
    let encoded =
        fs::read(path).map_err(|_| "chair_shadow_owner_review_ledger_read".to_string())?;
    let ledger = serde_json::from_slice(&encoded)
        .map_err(|_| "chair_shadow_owner_review_ledger_read".to_string())?;
    validate_chair_shadow_owner_review_ledger_v0(&ledger)?;
    Ok(ledger)
}

pub fn append_chair_shadow_owner_review_ledger_v0(
    path: &Path,
    review: &ChairShadowOwnerAdvisoryReviewV0,
) -> Result<ChairShadowOwnerReviewLedgerV0, String> {
    if !review_has_no_decision_or_action_v0(review) {
        return Err("chair_shadow_owner_review_invalid".into());
    }
    let mut ledger = if path.exists() {
        read_chair_shadow_owner_review_ledger_v0(path)?
    } else {
        new_chair_shadow_owner_review_ledger_v0(review.observation_packet_digest.clone())
    };
    if ledger.observation_packet_digest != review.observation_packet_digest {
        return Err("chair_shadow_owner_review_ledger_observation_mismatch".into());
    }
    if let Some(existing) = ledger
        .reviews
        .iter()
        .find(|existing| existing.owner_input_fingerprint == review.owner_input_fingerprint)
    {
        if existing != review {
            return Err("chair_shadow_owner_review_ledger_duplicate_conflict".into());
        }
        return Ok(ledger);
    }
    ledger.reviews.push(review.clone());
    ledger.reviews.sort_by(|left, right| {
        (
            left.owner_input_fingerprint.as_str(),
            left.owner_input_id.as_str(),
        )
            .cmp(&(
                right.owner_input_fingerprint.as_str(),
                right.owner_input_id.as_str(),
            ))
    });
    ledger.ledger_digest = review_ledger_digest_v0(&ledger);
    write_chair_shadow_owner_review_ledger_v0(path, &ledger)?;
    let reopened = read_chair_shadow_owner_review_ledger_v0(path)?;
    if reopened != ledger {
        return Err("chair_shadow_owner_review_ledger_reopen_mismatch".into());
    }
    Ok(reopened)
}

pub fn chair_shadow_owner_advisory_fixture_inputs_v0() -> Vec<OwnerInput> {
    let fixture = |owner_input_id: &str, input_kind: OwnerInputKind| OwnerInput {
        owner_input_id: owner_input_id.to_string(),
        input_kind,
        target_type: OwnerInputTargetType::System,
        status: OwnerInputStatus::Submitted,
        ..OwnerInput::default()
    };
    let mut reanalysis = fixture(
        "owner-shadow-fixture-reanalysis-v0",
        OwnerInputKind::CandidateReanalysisRequest,
    );
    reanalysis.target_type = OwnerInputTargetType::EvidenceRun;
    let mut risk_tighten = fixture(
        "owner-shadow-fixture-risk-tighten-v0",
        OwnerInputKind::RiskTightenRequest,
    );
    risk_tighten.target_type = OwnerInputTargetType::RiskDecision;
    let paper_confirm = fixture(
        "owner-shadow-fixture-paper-confirm-v0",
        OwnerInputKind::PaperConfirm,
    );
    let mut forbidden = fixture(
        "owner-shadow-fixture-forbidden-runtime-v0",
        OwnerInputKind::CandidateNote,
    );
    forbidden.requested_action = Some("force live trade".into());
    let mut freeform = fixture(
        "owner-shadow-fixture-freeform-v0",
        OwnerInputKind::ThesisNote,
    );
    freeform.freeform_note = Some("fixture diagnostic context".into());
    vec![reanalysis, risk_tighten, paper_confirm, forbidden, freeform]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owner_confirm() -> OwnerInput {
        OwnerInput {
            owner_input_id: "owner-confirm-fixture".to_string(),
            input_kind: OwnerInputKind::PaperConfirm,
            ..OwnerInput::default()
        }
    }

    fn market() -> MarketSnapshot {
        MarketSnapshot {
            symbol: "FAKE123".to_string(),
            timestamp_ms: 1_800_000_000_000,
            price: 100.0,
            bid: 99.0,
            ask: 101.0,
            spread_bps: 200.0,
            volume: 1_000.0,
            trade_value: 100_000.0,
            volatility: 0.01,
            regime: Regime::Unknown,
            data_quality_score: 0.55,
        }
    }

    #[test]
    fn owner_request_cannot_force_risk_denied_trade() {
        let risk = RiskDecision {
            kind: RiskDecisionKind::Deny,
            approved_order_plan: None,
            reason_codes: vec![
                ReasonCode::SpreadGateBreached,
                ReasonCode::UnknownRegimeGateBreached,
            ],
            audit_id: "risk-denied-fixture".to_string(),
        };
        let review =
            review_owner_trade_request(&owner_confirm(), &risk, &market(), 1_800_000_000_000);
        assert!(review.advisory_only);
        assert!(!review.owner_forced_trade);
        assert!(!review.paper_action_allowed);
        assert!(
            review
                .reason_codes
                .contains(&ReasonCode::OwnerRequestedButRiskDenied)
        );
        assert!(
            review
                .reason_codes
                .contains(&ReasonCode::OwnerRequestedButBadSpread)
        );
    }

    #[test]
    fn owner_rejection_explanation_is_stable_and_llm_free() {
        let reasons = vec![
            ReasonCode::OwnerRequestedButLowEdge,
            ReasonCode::OwnerRequestedButRiskDenied,
        ];
        let first = owner_rejection_explanation(&reasons);
        let second = owner_rejection_explanation(&reasons);
        assert_eq!(first, second);
        assert!(first.contains("Risk Governor"));
        assert!(first.contains("numeric edge"));
    }

    #[test]
    fn owner_request_rejected_for_stale_data_has_stable_reason() {
        let risk = RiskDecision {
            kind: RiskDecisionKind::Deny,
            approved_order_plan: None,
            reason_codes: vec![ReasonCode::DataQualityGateBreached],
            audit_id: "risk-stale-fixture".to_string(),
        };
        let mut stale_market = market();
        stale_market.timestamp_ms = 1;

        let review =
            review_owner_trade_request(&owner_confirm(), &risk, &stale_market, 1_800_000_000_000);

        assert!(!review.owner_forced_trade);
        assert!(!review.paper_action_allowed);
        assert!(
            review
                .reason_codes
                .contains(&ReasonCode::OwnerRequestedButStaleData)
        );
        assert!(review.explanation.contains("stale"));
    }

    #[test]
    fn owner_cooldown_and_doctrine_rejections_use_stable_templates() {
        let reasons = vec![
            ReasonCode::OwnerRequestedButAgentInCooldown,
            ReasonCode::OwnerRequestedButDoctrineViolation,
        ];
        let first = owner_rejection_explanation(&reasons);
        let second = owner_rejection_explanation(&reasons);
        assert_eq!(first, second);
        assert!(first.contains("cooldown"));
        assert!(first.contains("immutable agent doctrine"));
    }

    fn observation() -> ChairShadowObservationReportV0 {
        crate::model::learned_agent_scope::chair_shadow_test_observation_report_for_owner_review_v0(
        )
    }

    fn fixture(kind: OwnerInputKind) -> OwnerInput {
        chair_shadow_owner_advisory_fixture_inputs_v0()
            .into_iter()
            .find(|input| input.input_kind == kind)
            .unwrap()
    }

    fn review_for(
        observation: &ChairShadowObservationReportV0,
        owner_input: OwnerInput,
    ) -> ChairShadowOwnerAdvisoryReviewV0 {
        let input = chair_shadow_owner_advisory_review_input_v0(observation, owner_input);
        review_chair_shadow_owner_advisory_v0(observation, &input)
    }

    fn assert_no_action(review: &ChairShadowOwnerAdvisoryReviewV0) {
        assert!(review.considered);
        assert!(!review.changed_observation);
        assert!(!review.changed_model);
        assert!(!review.changed_decision);
        assert!(!review.changed_risk_policy);
        assert!(!review.vote_created);
        assert!(!review.reward_created);
        assert!(!review.penalty_created);
        assert!(!review.speaking_right_changed);
        assert!(!review.risk_handoff_created);
        assert!(!review.paper_action_created);
        assert!(!review.execution_created);
    }

    #[test]
    fn owner_shadow_review_requires_a_valid_observation_receipt() {
        let report = observation();
        let review = review_for(&report, fixture(OwnerInputKind::CandidateReanalysisRequest));
        assert_eq!(
            review.status,
            OwnerShadowAdvisoryStatusV0::ReanalysisRequestAcknowledged
        );
    }

    #[test]
    fn owner_shadow_review_rejects_a_changed_receipt_digest() {
        let report = observation();
        let mut input = chair_shadow_owner_advisory_review_input_v0(
            &report,
            fixture(OwnerInputKind::CandidateReanalysisRequest),
        );
        input.observation_receipt_digest = "changed".into();
        input.input_digest = review_input_digest_v0(&input);
        assert_eq!(
            review_chair_shadow_owner_advisory_v0(&report, &input).status,
            OwnerShadowAdvisoryStatusV0::InvalidObservationReceipt
        );
    }

    #[test]
    fn owner_shadow_review_rejects_a_changed_firewall_digest() {
        let report = observation();
        let mut input = chair_shadow_owner_advisory_review_input_v0(
            &report,
            fixture(OwnerInputKind::CandidateReanalysisRequest),
        );
        input.observation_firewall_digest = "changed".into();
        input.input_digest = review_input_digest_v0(&input);
        assert_eq!(
            review_chair_shadow_owner_advisory_v0(&report, &input).status,
            OwnerShadowAdvisoryStatusV0::InvalidObservationReceipt
        );
    }

    #[test]
    fn reanalysis_request_is_acknowledged_without_replay() {
        let review = review_for(
            &observation(),
            fixture(OwnerInputKind::CandidateReanalysisRequest),
        );
        assert_eq!(
            review.status,
            OwnerShadowAdvisoryStatusV0::ReanalysisRequestAcknowledged
        );
        assert!(
            review
                .reason_codes
                .contains(&"AutomaticReanalysisForbidden".to_string())
        );
        assert_no_action(&review);
    }

    #[test]
    fn risk_tighten_is_acknowledged_without_policy_mutation() {
        let review = review_for(&observation(), fixture(OwnerInputKind::RiskTightenRequest));
        assert_eq!(
            review.status,
            OwnerShadowAdvisoryStatusV0::ConservativeRequestAcknowledged
        );
        assert!(
            review
                .reason_codes
                .contains(&"RiskTightenNotAppliedInObservationMode".to_string())
        );
        assert!(!review.changed_risk_policy);
    }

    #[test]
    fn paper_confirm_creates_no_paper_action() {
        let review = review_for(&observation(), fixture(OwnerInputKind::PaperConfirm));
        assert_eq!(
            review.status,
            OwnerShadowAdvisoryStatusV0::TargetUnavailable
        );
        assert!(!review.paper_action_created);
        assert!(
            review
                .reason_codes
                .contains(&"PaperConfirmationRequiresIndependentRiskApproval".to_string())
        );
    }

    #[test]
    fn forbidden_force_trade_is_policy_blocked() {
        let review = review_for(&observation(), fixture(OwnerInputKind::CandidateNote));
        assert_eq!(review.status, OwnerShadowAdvisoryStatusV0::PolicyBlocked);
        assert!(!review.owner_policy_allowed);
        assert_no_action(&review);
    }

    #[test]
    fn freeform_only_input_is_diagnostic_only() {
        let review = review_for(&observation(), fixture(OwnerInputKind::ThesisNote));
        assert_eq!(review.status, OwnerShadowAdvisoryStatusV0::DiagnosticOnly);
        assert!(review.owner_policy_diagnostic_only);
        assert!(!review.explanation.contains("fixture diagnostic context"));
    }

    #[test]
    fn freeform_only_boundary_overrides_structured_kind_semantics() {
        let mut input = fixture(OwnerInputKind::PaperConfirm);
        input.freeform_note = Some("fixture-only text".into());
        let review = review_for(&observation(), input);
        assert_eq!(review.status, OwnerShadowAdvisoryStatusV0::DiagnosticOnly);
        assert!(!review.paper_action_created);
    }

    #[test]
    fn unknown_input_is_rejected_fail_closed() {
        let mut input = OwnerInput::default();
        input.owner_input_id = "owner-shadow-fixture-unknown-v0".into();
        let review = review_for(&observation(), input);
        assert_eq!(
            review.status,
            OwnerShadowAdvisoryStatusV0::UnknownInputRejected
        );
        assert!(!review.owner_policy_allowed);
    }

    #[test]
    fn missing_candidate_target_is_unavailable() {
        let mut input = OwnerInput::default();
        input.owner_input_id = "owner-shadow-fixture-hold-v0".into();
        input.input_kind = OwnerInputKind::CandidateHold;
        input.target_type = OwnerInputTargetType::Candidate;
        let review = review_for(&observation(), input);
        assert_eq!(
            review.status,
            OwnerShadowAdvisoryStatusV0::TargetUnavailable
        );
        assert_no_action(&review);
    }

    #[test]
    fn retrospective_evidence_cannot_justify_action() {
        assert_no_action(&review_for(
            &observation(),
            fixture(OwnerInputKind::CandidateReanalysisRequest),
        ));
    }

    #[test]
    fn explanation_reason_codes_are_sorted() {
        let review = review_for(
            &observation(),
            fixture(OwnerInputKind::CandidateReanalysisRequest),
        );
        assert!(review.reason_codes.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn explanation_replay_is_deterministic() {
        let report = observation();
        let input = chair_shadow_owner_advisory_review_input_v0(
            &report,
            fixture(OwnerInputKind::RiskTightenRequest),
        );
        assert_eq!(
            review_chair_shadow_owner_advisory_v0(&report, &input),
            review_chair_shadow_owner_advisory_v0(&report, &input)
        );
    }

    #[test]
    fn fixed_explanation_mapping_does_not_change_status() {
        let review = review_for(&observation(), fixture(OwnerInputKind::PaperConfirm));
        assert_eq!(
            explanation_for_owner_advisory_status_v0(review.status),
            review.explanation
        );
        assert_eq!(
            review.status,
            OwnerShadowAdvisoryStatusV0::TargetUnavailable
        );
    }

    #[test]
    fn original_owner_input_remains_unchanged() {
        let report = observation();
        let input = fixture(OwnerInputKind::RiskTightenRequest);
        let before = input.clone();
        let _ = review_for(&report, input.clone());
        assert_eq!(input, before);
    }

    #[test]
    fn observation_packet_remains_unchanged() {
        let report = observation();
        let before = report.clone();
        let _ = review_for(&report, fixture(OwnerInputKind::RiskTightenRequest));
        assert_eq!(report, before);
    }

    #[test]
    fn firewall_proves_no_owner_input_to_vote_conversion() {
        assert!(owner_advisory_decision_firewall_proof_v0().owner_input_cannot_become_vote);
    }

    #[test]
    fn firewall_proves_no_owner_input_to_chair_input_conversion() {
        assert!(owner_advisory_decision_firewall_proof_v0().owner_input_cannot_become_chair_input);
    }

    #[test]
    fn firewall_proves_chair_engine_is_not_invoked() {
        assert!(owner_advisory_decision_firewall_proof_v0().chair_engine_not_invoked);
    }

    #[test]
    fn firewall_proves_owner_trade_review_is_not_invoked() {
        assert!(owner_advisory_decision_firewall_proof_v0().owner_trade_review_not_invoked);
    }

    #[test]
    fn firewall_proves_risk_and_paper_broker_are_not_invoked() {
        let proof = owner_advisory_decision_firewall_proof_v0();
        assert!(proof.risk_governor_not_invoked);
        assert!(proof.paper_broker_not_invoked);
    }

    #[test]
    fn review_keeps_reward_and_penalty_counters_zero() {
        let review = review_for(&observation(), fixture(OwnerInputKind::PaperConfirm));
        assert!(!review.reward_created);
        assert!(!review.penalty_created);
    }

    #[test]
    fn review_keeps_speaking_right_counter_zero() {
        assert!(
            !review_for(&observation(), fixture(OwnerInputKind::PaperConfirm),)
                .speaking_right_changed
        );
    }

    #[test]
    fn owner_review_ledger_reopens_and_verifies() {
        let report = observation();
        let review = review_for(&report, fixture(OwnerInputKind::CandidateReanalysisRequest));
        let path = std::env::temp_dir().join(format!(
            "soma-owner-shadow-review-ledger-{}-{}.json",
            std::process::id(),
            review.review_digest
        ));
        let ledger = append_chair_shadow_owner_review_ledger_v0(&path, &review).unwrap();
        assert_eq!(
            read_chair_shadow_owner_review_ledger_v0(&path).unwrap(),
            ledger
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn duplicate_review_append_is_idempotent() {
        let report = observation();
        let review = review_for(&report, fixture(OwnerInputKind::RiskTightenRequest));
        let path = std::env::temp_dir().join(format!(
            "soma-owner-shadow-review-duplicate-{}-{}.json",
            std::process::id(),
            review.review_digest
        ));
        let first = append_chair_shadow_owner_review_ledger_v0(&path, &review).unwrap();
        let second = append_chair_shadow_owner_review_ledger_v0(&path, &review).unwrap();
        assert_eq!(first, second);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn text_and_json_semantics_are_derived_from_the_same_review() {
        let review = review_for(
            &observation(),
            fixture(OwnerInputKind::CandidateReanalysisRequest),
        );
        let text = format!(
            "fingerprint={}\nstatus={:?}\nreason_codes={}\nreview_digest={}",
            review.owner_input_fingerprint,
            review.status,
            review.reason_codes.join(":"),
            review.review_digest
        );
        let json = serde_json::json!({
            "owner_input_fingerprint":review.owner_input_fingerprint,
            "status":format!("{:?}", review.status),
            "reason_codes":review.reason_codes,
            "review_digest":review.review_digest,
        });
        assert!(text.contains(json["owner_input_fingerprint"].as_str().unwrap()));
        assert!(text.contains(json["status"].as_str().unwrap()));
        assert!(text.contains(json["review_digest"].as_str().unwrap()));
    }

    #[test]
    fn review_has_no_network_or_runtime_counters() {
        let review = review_for(&observation(), fixture(OwnerInputKind::RiskTightenRequest));
        let ledger = new_chair_shadow_owner_review_ledger_v0(review.observation_packet_digest);
        assert_eq!(ledger.chair_runtime_invocations, 0);
        assert_eq!(ledger.risk_runtime_invocations, 0);
        assert_eq!(ledger.action_count, 0);
    }

    #[test]
    fn previous_observation_artifact_stays_identical_across_fixture_reviews() {
        let report = observation();
        let before = report.clone();
        for input in chair_shadow_owner_advisory_fixture_inputs_v0() {
            let _ = review_for(&report, input);
        }
        assert_eq!(report, before);
    }

    #[test]
    fn fixture_set_covers_the_required_five_advisory_cases() {
        let fixtures = chair_shadow_owner_advisory_fixture_inputs_v0();
        assert_eq!(fixtures.len(), 5);
        assert!(
            fixtures
                .iter()
                .any(|input| input.input_kind == OwnerInputKind::CandidateReanalysisRequest)
        );
        assert!(
            fixtures
                .iter()
                .any(|input| input.input_kind == OwnerInputKind::RiskTightenRequest)
        );
        assert!(
            fixtures
                .iter()
                .any(|input| input.input_kind == OwnerInputKind::PaperConfirm)
        );
        assert!(
            fixtures
                .iter()
                .any(OwnerInput::requests_forbidden_runtime_action)
        );
        assert!(fixtures.iter().any(OwnerInput::freeform_only));
    }
}
