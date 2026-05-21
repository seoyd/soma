mod support;

use soma_zero::{
    RealFullWorkspaceDiagnosticPassStatus, RealWorkspaceTimeoutAttributionConfig,
    Sprint93TimeoutAttributionRunner,
};
use support::sprint69_support as sprint;

fn config(name: &str) -> RealWorkspaceTimeoutAttributionConfig {
    sprint::sprint93_config_from_example("soma_real_full_diagnostic_pass.toml", name)
}

#[test]
fn full_diagnostic_pass_stays_blocked_without_claiming_acceptance() {
    let report = Sprint93TimeoutAttributionRunner::default()
        .run_real_full_workspace_diagnostic_pass(&config("real-full-diagnostic"))
        .expect("report");
    assert_eq!(
        report.status,
        RealFullWorkspaceDiagnosticPassStatus::DiagnosticFullStillBlocked
    );
    assert!(!report.finished);
    assert_eq!(report.passed, None);
}
