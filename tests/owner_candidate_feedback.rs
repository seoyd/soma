use soma_zero::{
    OwnerFeedbackDecisionEffect, OwnerInput, OwnerInputKind, OwnerInputStatus,
    OwnerInputTargetType, build_owner_candidate_feedback, validate_owner_input,
};

fn input(kind: OwnerInputKind) -> OwnerInput {
    OwnerInput {
        owner_input_id: format!("feedback-{kind:?}"),
        timestamp_ms: Some(1715694000000),
        owner_id: Some("owner-local".to_string()),
        input_kind: kind,
        target_type: OwnerInputTargetType::Candidate,
        target_id: Some("cand-feedback".to_string()),
        symbol: Some("NVDA".to_string()),
        market: Some("USEquity".to_string()),
        freeform_note: None,
        structured_payload: None,
        requested_action: Some("review".to_string()),
        status: OwnerInputStatus::Submitted,
        reason_codes: vec![],
    }
}

#[test]
fn feedback_effects_map_to_owner_actions() {
    let note = build_owner_candidate_feedback(
        &input(OwnerInputKind::CandidateNote),
        &validate_owner_input(&input(OwnerInputKind::CandidateNote)),
    )
    .expect("note feedback");
    assert_eq!(
        note.affects_decision,
        OwnerFeedbackDecisionEffect::NoDirectEffect
    );

    let hold = build_owner_candidate_feedback(
        &input(OwnerInputKind::CandidateHold),
        &validate_owner_input(&input(OwnerInputKind::CandidateHold)),
    )
    .expect("hold feedback");
    assert_eq!(
        hold.affects_decision,
        OwnerFeedbackDecisionEffect::CandidateHeld
    );

    let dismiss = build_owner_candidate_feedback(
        &input(OwnerInputKind::CandidateDismiss),
        &validate_owner_input(&input(OwnerInputKind::CandidateDismiss)),
    )
    .expect("dismiss feedback");
    assert_eq!(
        dismiss.affects_decision,
        OwnerFeedbackDecisionEffect::CandidateDismissed
    );

    let reanalysis = build_owner_candidate_feedback(
        &input(OwnerInputKind::CandidateReanalysisRequest),
        &validate_owner_input(&input(OwnerInputKind::CandidateReanalysisRequest)),
    )
    .expect("reanalysis feedback");
    assert_eq!(
        reanalysis.affects_decision,
        OwnerFeedbackDecisionEffect::ChairReanalysisRequested
    );

    let risk = build_owner_candidate_feedback(
        &input(OwnerInputKind::RiskTightenRequest),
        &validate_owner_input(&input(OwnerInputKind::RiskTightenRequest)),
    )
    .expect("risk feedback");
    assert_eq!(
        risk.affects_decision,
        OwnerFeedbackDecisionEffect::RiskMoreConservativeRequested
    );

    let paper = build_owner_candidate_feedback(
        &input(OwnerInputKind::PaperConfirm),
        &validate_owner_input(&input(OwnerInputKind::PaperConfirm)),
    )
    .expect("paper feedback");
    assert_eq!(
        paper.affects_decision,
        OwnerFeedbackDecisionEffect::PaperConfirmed
    );
    assert_eq!(paper.fingerprint(), paper.fingerprint());
}
