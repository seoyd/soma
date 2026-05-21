use soma_zero::{
    OwnerInput, OwnerInputKind, OwnerInputStatus, OwnerInputTargetType, OwnerPolicyConstraintKind,
    ReasonCode, validate_owner_input,
};

fn input_with_action(kind: OwnerInputKind, requested_action: &str) -> OwnerInput {
    OwnerInput {
        owner_input_id: format!("policy-{kind:?}"),
        timestamp_ms: Some(1715692000000),
        owner_id: Some("owner-local".to_string()),
        input_kind: kind,
        target_type: OwnerInputTargetType::Candidate,
        target_id: Some("cand-101".to_string()),
        symbol: Some("005930.KS".to_string()),
        market: Some("KoreanEquity".to_string()),
        freeform_note: None,
        structured_payload: None,
        requested_action: Some(requested_action.to_string()),
        status: OwnerInputStatus::Submitted,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

#[test]
fn hard_policy_constraints_block_unsafe_actions() {
    let cases = [
        (
            "override risk governor",
            OwnerPolicyConstraintKind::CannotForceTrade,
        ),
        (
            "enable live trading",
            OwnerPolicyConstraintKind::CannotEnableLiveTrading,
        ),
        (
            "enable broker api",
            OwnerPolicyConstraintKind::CannotEnableBrokerAPI,
        ),
        (
            "loosen hard veto",
            OwnerPolicyConstraintKind::CannotLoosenHardVeto,
        ),
    ];
    for (requested_action, _expected_kind) in cases {
        let validation = validate_owner_input(&input_with_action(
            OwnerInputKind::PaperConfirm,
            requested_action,
        ));
        assert!(!validation.allowed);
        assert!(!validation.blocked_constraints.is_empty());
    }
}

#[test]
fn conservative_and_candidate_actions_are_allowed() {
    for kind in [
        OwnerInputKind::RiskTightenRequest,
        OwnerInputKind::CandidateDismiss,
        OwnerInputKind::CandidateHold,
    ] {
        let validation = validate_owner_input(&input_with_action(kind, "safe-structured-action"));
        assert!(validation.allowed);
    }
}

#[test]
fn policy_validation_is_deterministic() {
    let input = input_with_action(OwnerInputKind::CandidateDismiss, "dismiss_candidate");
    let first = validate_owner_input(&input);
    let second = validate_owner_input(&input);
    assert_eq!(first, second);
}
