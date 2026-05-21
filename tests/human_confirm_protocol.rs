use soma_zero::{
    HumanConfirmProtocolConfig, HumanConfirmState, OwnerInputKind,
    build_human_confirm_protocol_report, evaluate_human_confirm_transition,
};

#[test]
fn protocol_allows_and_blocks_expected_transitions() {
    let protocol = HumanConfirmProtocolConfig::default();
    assert!(
        evaluate_human_confirm_transition(
            &protocol,
            HumanConfirmState::PendingReview,
            OwnerInputKind::MarkReviewed,
        )
        .allowed
    );
    assert!(
        evaluate_human_confirm_transition(
            &protocol,
            HumanConfirmState::PendingReview,
            OwnerInputKind::CandidateDismiss,
        )
        .allowed
    );
    assert!(
        evaluate_human_confirm_transition(
            &protocol,
            HumanConfirmState::PendingReview,
            OwnerInputKind::CandidateHold,
        )
        .allowed
    );
    assert!(
        evaluate_human_confirm_transition(
            &protocol,
            HumanConfirmState::HumanConfirmRequired,
            OwnerInputKind::PaperConfirm,
        )
        .allowed
    );
    assert!(
        !evaluate_human_confirm_transition(
            &protocol,
            HumanConfirmState::RiskBlocked,
            OwnerInputKind::PaperConfirm,
        )
        .allowed
    );
    assert!(
        !evaluate_human_confirm_transition(
            &protocol,
            HumanConfirmState::NoTrade,
            OwnerInputKind::PaperConfirm,
        )
        .allowed
    );
}

#[test]
fn protocol_has_no_execute_order_transition_and_is_deterministic() {
    let report = build_human_confirm_protocol_report(&HumanConfirmProtocolConfig::default());
    assert!(
        report
            .forbidden_actions_summary
            .contains(&"ExecuteOrder".to_string())
    );
    let second = build_human_confirm_protocol_report(&HumanConfirmProtocolConfig::default());
    assert_eq!(report, second);
}
