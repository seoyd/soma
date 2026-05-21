mod support;

use support::sprint108_support::{read_fixture, run_sprint108};

#[test]
fn retired_target_safety_audit_matches_expected_fixture() {
    let bundle = run_sprint108(
        "soma_retired_target_safety_audit_v2.toml",
        "retired-target-safety-audit-v2",
    );
    let actual =
        serde_json::to_value(&bundle.retired_target_safety_audit_report_v2).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint108_data/retired_target_safety_audit_expected.json");
    assert_eq!(actual, expected);
    assert!(
        !bundle
            .retired_target_safety_audit_report_v2
            .high_risk_target_retired
    );
    assert!(
        !bundle
            .retired_target_safety_audit_report_v2
            .safety_sentinel_retired
    );
    assert_eq!(
        bundle.retired_target_safety_audit_report_v2.audit_status,
        "RetiredTargetSafetyReady"
    );
}

#[test]
fn manual_high_risk_or_sentinel_retirement_is_detectable() {
    let bundle = run_sprint108(
        "soma_retired_target_safety_audit_v2.toml",
        "retired-target-safety-audit-v2-detect",
    );
    let mut report = bundle.retired_target_safety_audit_report_v2;
    report.high_risk_target_retired = true;
    report.safety_sentinel_retired = true;
    assert!(report.high_risk_target_retired);
    assert!(report.safety_sentinel_retired);
}
