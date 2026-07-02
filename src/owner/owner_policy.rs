use serde::{Deserialize, Serialize};

use crate::core::{
    MarketSnapshot, ReasonCode, Regime, RiskDecision, RiskDecisionKind, stable_reason_codes,
};

use super::{OwnerInput, OwnerInputKind};

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
}
