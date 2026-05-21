mod support;

use soma_zero::{
    RealWorkspaceTimeoutAttributionConfig, Sprint93TimeoutAttributionRunner,
    UnknownTimeoutClosureStatus,
};
use support::sprint69_support as sprint;

fn config(name: &str) -> RealWorkspaceTimeoutAttributionConfig {
    sprint::sprint93_config_from_example("soma_unknown_timeout_closure.toml", name)
}

#[test]
fn unknown_timeout_closure_reports_closed_unknowns() {
    let report = Sprint93TimeoutAttributionRunner::default()
        .run_unknown_timeout_closure(&config("unknown-timeout-closure"))
        .expect("report");
    assert_eq!(
        report.closure_status,
        UnknownTimeoutClosureStatus::UnknownTimeoutClosed
    );
    assert!(report.remaining_unknowns.is_empty());
}
