#[path = "support/sprint58_support.rs"]
mod sprint58_support;

use soma_zero::{SecretRedactionAuditRunner, SecretRedactionStatus};

#[test]
fn secret_redaction_audit_passes_safe_fixture() {
    let out = sprint58_support::output_dir("secret-redaction-safe");
    let report = SecretRedactionAuditRunner::default()
        .run(&sprint58_support::secret_audit_config(
            &out,
            vec![
                sprint58_support::sprint58_data_path("secret_redaction_sample_safe.txt")
                    .display()
                    .to_string(),
            ],
        ))
        .expect("audit");
    assert_eq!(report.redaction_status, SecretRedactionStatus::Passed);
}

#[test]
fn secret_redaction_audit_detects_unsafe_fixture() {
    let out = sprint58_support::output_dir("secret-redaction-unsafe");
    let report = SecretRedactionAuditRunner::default()
        .run(&sprint58_support::secret_audit_config(
            &out,
            vec![
                sprint58_support::sprint58_data_path("secret_redaction_sample_unsafe.txt")
                    .display()
                    .to_string(),
            ],
        ))
        .expect("audit");
    assert!(matches!(
        report.redaction_status,
        SecretRedactionStatus::FailedTokenLeak
            | SecretRedactionStatus::FailedAccountField
            | SecretRedactionStatus::FailedOrderField
    ));
    assert!(report.account_like_fields_detected);
    assert!(report.order_like_fields_detected);
}

#[test]
fn secret_redaction_audit_does_not_flag_reason_code_identifiers_as_tokens() {
    let out = sprint58_support::output_dir("secret-redaction-reason-code");
    let report = SecretRedactionAuditRunner::default()
        .run(&sprint58_support::secret_audit_config(
            &out,
            vec![
                sprint58_support::sprint58_data_path("operational_runbook_v2_expected.json")
                    .display()
                    .to_string(),
            ],
        ))
        .expect("audit");
    assert_eq!(report.redaction_status, SecretRedactionStatus::Passed);
}
