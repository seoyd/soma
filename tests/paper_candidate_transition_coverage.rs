mod support;

use support::sprint105_support::run_sprint105;

#[test]
fn paper_candidate_transition_coverage_is_complete_and_non_live() {
    let bundle = run_sprint105(
        "soma_paper_candidate_transition_coverage.toml",
        "paper_candidate_transition_coverage",
    );
    let report = &bundle.paper_candidate_transition_coverage_report;
    assert_eq!(report.total_states, 10);
    assert!(report.reachable_or_explained_states >= 9);
    assert_eq!(report.unsafe_transition_count, 0);
    assert!(
        bundle
            .paper_candidate_gate_completeness_report
            .promotion_gate_present
    );
    assert!(
        bundle
            .paper_candidate_gate_completeness_report
            .rejection_gate_present
    );
    assert!(
        bundle
            .paper_candidate_gate_completeness_report
            .watchlist_gate_present
    );
    assert!(
        bundle
            .paper_candidate_gate_completeness_report
            .no_trade_gate_present
    );
    assert!(
        bundle
            .paper_candidate_gate_completeness_report
            .risk_denied_gate_present
    );
}
