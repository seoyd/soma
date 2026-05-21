mod support;

use support::sprint102_support::run_sprint102;

#[test]
fn risk_governor_handoff_keeps_execution_forbidden() {
    let bundle = run_sprint102("soma_risk_governor_paper_handoff.toml", "sprint102-risk");
    let handoff = &bundle.risk_governor_paper_handoff_report;
    assert!(!handoff.broker_execution_allowed);
    assert!(!handoff.live_execution_allowed);
    assert!(!handoff.risk_checks.is_empty());
    assert!(
        !bundle
            .no_trade_risk_denied_committee_trace
            .no_trade_member_votes
            .is_empty()
    );
    assert!(
        !bundle
            .no_trade_risk_denied_committee_trace
            .risk_deny_member_votes
            .is_empty()
    );
}
