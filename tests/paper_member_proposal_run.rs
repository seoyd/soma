mod support;

use support::sprint102_support::run_sprint102;

#[test]
fn paper_member_proposal_run_counts_expected_outcomes() {
    let bundle = run_sprint102("soma_paper_member_proposal_run.toml", "sprint102-proposals");
    let run = &bundle.paper_only_member_proposal_run;
    assert_eq!(
        run.generated_proposals.len(),
        run.enter_long_count
            + run.enter_short_count
            + run.wait_count
            + run.no_trade_count
            + run.risk_deny_count
            + run.request_more_evidence_count
    );
    assert!(run.enter_long_count > 0);
    assert!(run.enter_short_count > 0);
    assert!(run.wait_count > 0);
    assert!(run.no_trade_count > 0);
    assert!(run.risk_deny_count > 0);
    assert!(run.request_more_evidence_count > 0);
    assert!(run.proposals_with_entry_timing > 0);
    assert!(run.proposals_with_risk_fields > 0);
    assert!(run.proposals_with_evidence_refs > 0);
    assert!(run.proposals_with_entry_timing <= run.generated_proposals.len());
    assert!(run.proposals_with_risk_fields <= run.generated_proposals.len());
    assert!(run.proposals_with_evidence_refs <= run.generated_proposals.len());
    assert!(
        bundle
            .proposal_outcome_expectation_trace
            .expectation_not_profit_claim
    );
}
