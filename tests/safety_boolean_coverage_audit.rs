mod support;

use soma_zero::Sprint105VerificationPatchClosureRunner;

#[test]
fn safety_boolean_audit_uses_actual_guards() {
    let bundle = support::sprint105_support::run_sprint105(
        "soma_safety_boolean_coverage_audit.toml",
        "safety_boolean_coverage_audit",
    );
    assert!(
        bundle
            .safety_boolean_coverage_audit_report
            .actual_guard_booleans_count
            > 0
    );
}

#[test]
fn safety_boolean_audit_detects_missing_guards() {
    let mut config = soma_zero::Sprint105VerificationPatchClosureConfig::default();
    config.preserve_safety_guards = false;
    let bundle = Sprint105VerificationPatchClosureRunner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        bundle.safety_boolean_coverage_audit_report.audit_status,
        "SafetyBooleanCoverageMissingGuards"
    );
    assert_eq!(
        bundle.safety_coverage_preservation_report_v21.safety_status,
        "SafetyCoverageRegressionV21"
    );
    assert!(
        !bundle
            .safety_coverage_preservation_report_v21
            .live_trading_guard_present
    );
}
