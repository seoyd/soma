mod common;

use soma_zero::{
    OwnerInput, OwnerInputKind, OwnerInputStatus, OwnerInputTargetType, ReasonCode,
    run_owner_input_validation, validate_owner_input,
};

fn base_input(kind: OwnerInputKind) -> OwnerInput {
    OwnerInput {
        owner_input_id: format!("{kind:?}-input"),
        timestamp_ms: Some(1715691000000),
        owner_id: Some("owner-local".to_string()),
        input_kind: kind,
        target_type: OwnerInputTargetType::Candidate,
        target_id: Some("cand-101".to_string()),
        symbol: Some("005930.KS".to_string()),
        market: Some("KoreanEquity".to_string()),
        freeform_note: None,
        structured_payload: Some(std::collections::BTreeMap::from([(
            "tag".to_string(),
            "structured".to_string(),
        )])),
        requested_action: Some("review".to_string()),
        status: OwnerInputStatus::Submitted,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

#[test]
fn owner_input_variants_can_be_constructed() {
    for kind in [
        OwnerInputKind::CandidateNote,
        OwnerInputKind::CandidateHold,
        OwnerInputKind::CandidateDismiss,
        OwnerInputKind::CandidateReanalysisRequest,
        OwnerInputKind::PaperConfirm,
        OwnerInputKind::RiskTightenRequest,
    ] {
        let input = base_input(kind);
        assert_eq!(input.input_kind, kind);
        assert!(!input.fingerprint().is_empty());
    }
}

#[test]
fn risk_loosen_request_is_diagnostic_only() {
    let input = base_input(OwnerInputKind::RiskLoosenRequestDiagnosticOnly);
    let validation = validate_owner_input(&input);
    assert!(validation.diagnostic_only);
    assert!(validation.allowed);
    assert!(
        validation
            .reason_codes
            .contains(&ReasonCode::OwnerRiskLoosenDiagnosticOnly)
    );
}

#[test]
fn unknown_input_cannot_be_applied_and_freeform_is_no_direct_effect() {
    let mut input = base_input(OwnerInputKind::Unknown);
    input.structured_payload = None;
    input.freeform_note = Some("Owner freeform opinion stays audited only.".to_string());
    let validation = validate_owner_input(&input);
    assert!(!validation.allowed);
    assert!(validation.diagnostic_only);
    assert!(
        validation
            .reason_codes
            .contains(&ReasonCode::OwnerFreeformNoteNoDirectEffect)
    );
    assert!(
        validation
            .reason_codes
            .contains(&ReasonCode::OwnerInputUnknownRejected)
    );
}

#[test]
fn owner_input_rendering_is_deterministic() {
    let config = soma_zero::OwnerInputValidateConfig {
        owner_input: base_input(OwnerInputKind::CandidateNote),
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    let first = run_owner_input_validation(&config);
    let second = run_owner_input_validation(&config);
    assert_eq!(first.fingerprint, second.fingerprint);
    assert_eq!(first.to_text(), second.to_text());
}
