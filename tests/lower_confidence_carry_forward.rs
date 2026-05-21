mod support;

use support::sprint104_support::run_sprint104;

#[test]
fn lower_confidence_carry_forward_lists_warning_backed_candidates() {
    let bundle = run_sprint104(
        "soma_sprint104_dual_agent_paper_lifecycle.toml",
        "lower_confidence_carry_forward",
    );
    let policy = bundle.lower_confidence_carry_forward_policy;
    assert!(
        policy
            .warning_backed_candidates
            .contains(&"Wonyotti".to_string())
    );
    assert!(
        policy
            .warning_backed_candidates
            .contains(&"LarryWilliams".to_string())
    );
    assert!(
        policy
            .warning_backed_candidates
            .contains(&"ArthurHayes".to_string())
    );
    assert!(policy.carry_forward_allowed_for_paper);
    assert!(!policy.carry_forward_allowed_for_live);
}

#[test]
fn lower_confidence_reviews_keep_explicit_warning_backed_guards() {
    let bundle = run_sprint104(
        "soma_sprint104_dual_agent_paper_lifecycle.toml",
        "lower_confidence_reviews",
    );
    assert!(
        bundle
            .wonyotti_carry_forward_review
            .exact_return_claims_blocked
    );
    assert!(
        bundle
            .larry_williams_carry_forward_review
            .exact_numeric_rule_claims_downweighted
    );
    assert!(
        bundle
            .arthur_hayes_carry_forward_review
            .leverage_risk_guard_present
    );
}
