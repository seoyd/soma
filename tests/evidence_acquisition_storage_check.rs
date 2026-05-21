use soma_zero::{
    OfficialEvidenceAcquisitionPlan, ReasonCode, build_evidence_acquisition_storage_check,
};

#[test]
fn all_symbol_request_is_rejected() {
    let check = build_evidence_acquisition_storage_check(&OfficialEvidenceAcquisitionPlan {
        allow_all_symbols: true,
        ..OfficialEvidenceAcquisitionPlan::default()
    });

    assert!(!check.budget_ok);
    assert!(check.reason_codes.contains(&ReasonCode::DeniedByDefault));
}

#[test]
fn full_history_request_is_rejected() {
    let check = build_evidence_acquisition_storage_check(&OfficialEvidenceAcquisitionPlan {
        allow_full_history: true,
        ..OfficialEvidenceAcquisitionPlan::default()
    });

    assert!(!check.budget_ok);
    assert!(check.reason_codes.contains(&ReasonCode::FullHistoryDenied));
}

#[test]
fn max_rows_requests_and_bytes_are_enforced() {
    let check = build_evidence_acquisition_storage_check(&OfficialEvidenceAcquisitionPlan {
        max_rows_per_symbol: 501,
        max_requests: 11,
        max_total_bytes: 1024,
        ..OfficialEvidenceAcquisitionPlan::default()
    });

    assert!(!check.budget_ok);
    assert!(check.reason_codes.contains(&ReasonCode::RowLimitApplied));
    assert!(check.reason_codes.contains(&ReasonCode::BudgetExceeded));
}

#[test]
fn storage_check_never_schedules_canonical_deletion() {
    let check =
        build_evidence_acquisition_storage_check(&OfficialEvidenceAcquisitionPlan::default());

    assert!(
        !check
            .warnings
            .iter()
            .any(|warning| warning.contains("delete canonical"))
    );
}

#[test]
fn storage_check_is_deterministic() {
    let first = serde_json::to_string(&build_evidence_acquisition_storage_check(
        &OfficialEvidenceAcquisitionPlan::default(),
    ))
    .expect("first");
    let second = serde_json::to_string(&build_evidence_acquisition_storage_check(
        &OfficialEvidenceAcquisitionPlan::default(),
    ))
    .expect("second");

    assert_eq!(first, second);
}
