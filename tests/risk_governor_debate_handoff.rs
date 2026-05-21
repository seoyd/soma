mod support;

use soma_zero::RiskGovernorDebateHandoffStatus;
use support::sprint99_support::run_sprint99;

#[test]
fn risk_governor_debate_handoff_preserves_final_veto() {
    let bundle = run_sprint99(
        "soma_risk_governor_debate_handoff.toml",
        "risk-governor-debate-handoff",
    );
    let report = bundle.risk_governor_debate_handoff_report;
    assert_eq!(
        report.handoff_status,
        RiskGovernorDebateHandoffStatus::RiskHandoffReadyWithWarnings
    );
    assert_eq!(report.sessions_with_risk_handoff, 1);
    assert_eq!(report.bypass_attempt_count, 0);
    assert!(report.risk_governor_final_veto_confirmed);
}
