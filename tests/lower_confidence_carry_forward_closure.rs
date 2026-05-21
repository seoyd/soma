mod support;

use support::sprint105_support::run_sprint105;

#[test]
fn lower_confidence_carry_forward_stays_explicit() {
    let bundle = run_sprint105(
        "soma_lower_confidence_carry_forward_closure.toml",
        "lower_confidence_carry_forward_closure",
    );
    let report = &bundle.lower_confidence_carry_forward_closure_report;
    assert!(
        report
            .warning_backed_candidates
            .iter()
            .any(|value| value.contains("wonyotti"))
    );
    assert_eq!(report.silent_upgrade_count, 0);
    assert!(!report.live_activation_allowed);
    assert!(
        bundle
            .wonyotti_carry_forward_closure_report
            .remains_warning_backed
    );
    assert!(
        bundle
            .larry_williams_carry_forward_closure_report
            .exact_numeric_rule_claims_downweighted
    );
    assert!(
        bundle
            .arthur_hayes_carry_forward_closure_report
            .leverage_risk_guard_present
    );
}
