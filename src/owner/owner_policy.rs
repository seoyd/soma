use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

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
