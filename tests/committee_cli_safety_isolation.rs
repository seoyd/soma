mod support;

use soma_zero::{CommitteeCliSafetyIsolationReportStatus, Sprint88SevenBlockerRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn committee_cli_safety_stays_isolated_with_required_checks() {
    let config = sprint::sprint88_config_from_example(
        "soma_committee_cli_safety_isolation.toml",
        "committee-isolation",
    );
    let report = Sprint88SevenBlockerRecoveryRunner::default()
        .run_committee_cli_safety_isolation(&config)
        .expect("report");
    assert!(report.keep_separate);
    for check in [
        "help_text_research_only",
        "remote_path_rejection",
        "no_runtime_llm",
        "no_persona_expansion",
        "no_broker_order_account",
        "deterministic_help",
    ] {
        assert!(report.checks_preserved.contains(&check.to_string()));
    }
    assert_eq!(
        report.report_status,
        CommitteeCliSafetyIsolationReportStatus::CommitteeCliSafetyKeptIsolated
    );
}
