mod support;

use support::sprint110_support::run_sprint110;

#[test]
fn retired_target_safety_audit_v4_keeps_sentinels_outside_retirement_set() {
    let bundle = run_sprint110(
        "soma_retired_target_safety_audit_v4.toml",
        "retired-target-safety-audit-v4",
    );
    let report = bundle.retired_target_safety_audit_report_v4;
    assert_eq!(
        report.retired_targets,
        vec!["tests/shared_toml_builder_application_v1.rs".to_string()]
    );
    assert_eq!(
        report.cumulative_retired_targets,
        vec![
            "tests/shared_fixture_harness_expansion_plan_v2.rs".to_string(),
            "tests/shared_output_dir_helper_application_v1.rs".to_string(),
            "tests/shared_render_helper_application_v1.rs".to_string(),
            "tests/shared_toml_builder_application_v1.rs".to_string(),
        ]
    );
    assert!(report.validation_reconciled_before_retirement);
    assert!(!report.high_risk_target_retired);
    assert!(!report.safety_sentinel_retired);
    assert_eq!(report.audit_status, "RetiredTargetSafetyReady");
}
