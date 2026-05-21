mod support;

use soma_zero::{
    RealNoRunDiagnosticPassStatus, RealWorkspaceTimeoutAttributionConfig,
    Sprint93TimeoutAttributionRunner, WorkspaceTimeoutAttributionStatus,
};
use support::sprint69_support as sprint;

fn config(name: &str) -> RealWorkspaceTimeoutAttributionConfig {
    sprint::sprint93_config_from_example("soma_real_timeout_attribution.toml", name)
}

#[test]
fn real_timeout_attribution_reports_ready_timeout_and_keeps_full_gate_honest() {
    let report = Sprint93TimeoutAttributionRunner::default()
        .run_real_timeout_attribution(&config("real-timeout-attribution"))
        .expect("report");
    assert_eq!(
        report.attribution_status,
        WorkspaceTimeoutAttributionStatus::TimeoutAttributionReady
    );
    assert_eq!(
        report.diagnostic_no_run_status,
        RealNoRunDiagnosticPassStatus::DiagnosticNoRunPassed
    );
    assert_eq!(report.previous_full_status, "FullWorkspaceStillBlocked");
}
