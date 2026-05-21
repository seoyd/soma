mod support;

use support::sprint105_support::run_sprint105;

#[test]
fn risk_governor_batch_veto_warnings_close_without_bypass() {
    let bundle = run_sprint105(
        "soma_risk_governor_batch_veto_warning_closure.toml",
        "risk_governor_batch_veto_warning_closure",
    );
    assert_eq!(
        bundle
            .risk_governor_batch_veto_warning_closure_report
            .bypass_attempt_count,
        0
    );
    assert!(
        bundle
            .risk_governor_transition_completeness_report
            .completeness_status
            .contains("Complete")
    );
    assert_eq!(
        bundle
            .risk_governor_no_bypass_audit_v2
            .bypass_transition_count,
        0
    );
}
