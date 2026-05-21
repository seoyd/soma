mod support;

use support::sprint102_support::run_sprint102;

#[test]
fn debate_trigger_session_and_conflicts_are_recorded() {
    let bundle = run_sprint102("soma_group_debate_session.toml", "sprint102-debate");
    assert!(bundle.group_debate_trigger_report.debate_required);
    assert!(bundle.group_debate_session_report.support_entry_count > 0);
    assert!(bundle.group_debate_session_report.oppose_entry_count > 0);
    assert!(bundle.group_debate_session_report.wait_count > 0);
    assert!(bundle.group_debate_session_report.no_trade_count > 0);
    assert!(bundle.group_debate_session_report.risk_deny_count > 0);
    assert!(
        bundle
            .group_debate_session_report
            .request_more_evidence_count
            > 0
    );
    assert!(bundle.cross_group_debate_conflict_report.conflicts_detected >= 4);
}
