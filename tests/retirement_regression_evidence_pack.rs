#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::RetirementRegressionEvidencePackStatus;

#[test]
fn retirement_regression_pack_reports_ready_only_with_full_local_evidence() {
    let pack = support::run_retirement_pack("soma_retirement_regression_pack.toml");
    assert_eq!(
        pack.pack_status,
        RetirementRegressionEvidencePackStatus::RetirementEvidenceReady
    );
    assert!(pack.regression_evidence_present);
    assert!(pack.owner_reason_present);
    assert!(pack.supports_retirement);
}

#[test]
fn retirement_regression_pack_falls_back_when_owner_reason_is_missing() {
    let mut config =
        support::retirement_pack_config_from_example("soma_retirement_regression_pack.toml");
    config.owner_review_paths.clear();
    config.comparison_paths.clear();

    let pack = soma_zero::OfflineEvidenceAttachmentRunner::default()
        .run_retirement_regression_pack(&config)
        .expect("run without owner reason");
    assert_eq!(
        pack.pack_status,
        RetirementRegressionEvidencePackStatus::DiagnosticOnlySupported
    );
    assert!(!pack.supports_retirement);
    assert!(pack.supports_diagnostic_only);
}
