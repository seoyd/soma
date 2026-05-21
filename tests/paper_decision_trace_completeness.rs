mod support;

use soma_zero::PaperDecisionTraceCompletenessStatus;
use support::sprint99_support::run_sprint99;

#[test]
fn paper_decision_trace_completeness_is_full() {
    let bundle = run_sprint99(
        "soma_paper_decision_trace_completeness.toml",
        "paper-decision-trace-completeness",
    );
    let report = bundle.paper_decision_trace_completeness_report;
    assert_eq!(
        report.trace_status,
        PaperDecisionTraceCompletenessStatus::TraceComplete
    );
    assert_eq!(report.decisions_missing_trace, 0);
    assert_eq!(report.decisions_with_member_proposals, 1);
    assert_eq!(report.decisions_with_debate_session, 1);
    assert_eq!(report.decisions_with_chair_synthesis, 1);
    assert_eq!(report.decisions_with_risk_governor_decision, 1);
}
