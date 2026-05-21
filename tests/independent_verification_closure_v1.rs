mod support;

use support::sprint108_support::run_sprint108;

#[test]
fn independent_verification_closure_counts_fixed_findings() {
    let bundle = run_sprint108(
        "soma_independent_verification_closure_v1.toml",
        "independent-verification-closure-v1",
    );
    let report = bundle.independent_verification_closure_report_v1;
    assert!(report.verification_performed);
    assert_eq!(report.findings_fixed, 4);
    assert_eq!(report.findings_remaining, 0);
    assert_eq!(
        report.final_verification_status,
        "IndependentVerificationClosedWithWarnings"
    );
}

#[test]
fn independent_verification_closure_is_deterministic() {
    let left = run_sprint108(
        "soma_independent_verification_closure_v1.toml",
        "independent-verification-closure-v1-left",
    );
    let right = run_sprint108(
        "soma_independent_verification_closure_v1.toml",
        "independent-verification-closure-v1-right",
    );
    assert_eq!(
        serde_json::to_value(&left.independent_verification_closure_report_v1).expect("left"),
        serde_json::to_value(&right.independent_verification_closure_report_v1).expect("right")
    );
}
