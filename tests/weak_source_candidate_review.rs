mod support;

use support::sprint102_support::run_sprint102;

#[test]
fn weak_source_reviews_preserve_candidate_specific_guards() {
    let bundle = run_sprint102(
        "soma_weak_source_candidate_review.toml",
        "sprint102-weak-source",
    );
    let review = &bundle.weak_source_candidate_review_report;
    assert_eq!(review.weak_source_warning_count, 3);
    let candidate_ids = review
        .candidate_reviews
        .iter()
        .map(|review| review.candidate_id.as_str())
        .collect::<Vec<_>>();
    assert!(candidate_ids.contains(&"wonyotti"));
    assert!(candidate_ids.contains(&"larry-williams"));
    assert!(candidate_ids.contains(&"arthur-hayes"));
    assert!(
        bundle
            .wonyotti_evidence_hardening_report
            .exact_return_claims_blocked
    );
    assert!(
        bundle
            .larry_williams_evidence_hardening_report
            .exact_numeric_rule_claims_downweighted
    );
    assert!(
        bundle
            .arthur_hayes_evidence_hardening_report
            .leverage_risk_guard_present
    );
}
