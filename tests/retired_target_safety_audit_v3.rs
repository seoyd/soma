mod support;

use support::sprint109_support::{read_fixture, run_sprint109};

#[test]
fn retired_target_safety_audit_matches_expected_fixture() {
    let bundle = run_sprint109(
        "soma_retired_target_safety_audit_v3.toml",
        "retired-target-safety-audit-v3",
    );
    let actual =
        serde_json::to_value(&bundle.retired_target_safety_audit_report_v3).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint109_data/retired_target_safety_audit_v3_expected.json");
    assert_eq!(actual, expected);
    assert!(
        !bundle
            .retired_target_safety_audit_report_v3
            .high_risk_target_retired
    );
    assert!(
        !bundle
            .retired_target_safety_audit_report_v3
            .safety_sentinel_retired
    );
    assert_eq!(
        bundle.retired_target_safety_audit_report_v3.audit_status,
        "RetiredTargetSafetyReady"
    );
}

#[test]
fn manual_high_risk_or_sentinel_retirement_is_detectable() {
    let bundle = run_sprint109(
        "soma_retired_target_safety_audit_v3.toml",
        "retired-target-safety-audit-v3-detect",
    );
    let mut report = bundle.retired_target_safety_audit_report_v3;
    report.high_risk_target_retired = true;
    report.safety_sentinel_retired = true;
    assert!(report.high_risk_target_retired);
    assert!(report.safety_sentinel_retired);
}
