use soma_zero::{
    ReasonCode, ReasonCodeCompletenessStatus, audit_reason_codes, critical_reason_codes,
};

#[test]
fn known_reason_codes_include_required_critical_failures() {
    let codes = critical_reason_codes();

    assert!(codes.contains(&ReasonCode::MissingFile));
    assert!(codes.contains(&ReasonCode::RemotePathRejected));
    assert!(codes.contains(&ReasonCode::MissingAuth));
    assert!(codes.contains(&ReasonCode::DataQualityTooLow));
    assert!(codes.contains(&ReasonCode::SchemaMismatch));
    assert!(codes.contains(&ReasonCode::InvalidPrediction));
    assert!(codes.contains(&ReasonCode::BudgetExceeded));
    assert!(codes.contains(&ReasonCode::LiveModeDisabled));
}

#[test]
fn unknown_reason_code_is_detected() {
    let audit = audit_reason_codes(
        &[ReasonCode::MissingFile, ReasonCode::BudgetExceeded],
        &[String::from("CustomUnknownReason")],
        None,
    );

    assert_eq!(
        audit.completeness_status,
        ReasonCodeCompletenessStatus::UnknownCodesFound
    );
    assert!(
        audit
            .unknown_reason_codes
            .contains(&"CustomUnknownReason".to_string())
    );
}
