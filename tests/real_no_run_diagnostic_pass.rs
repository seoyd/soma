mod support;

use soma_zero::{
    RealNoRunDiagnosticPassStatus, RealWorkspaceTimeoutAttributionConfig,
    Sprint93TimeoutAttributionRunner,
};
use support::sprint69_support as sprint;

fn config(name: &str) -> RealWorkspaceTimeoutAttributionConfig {
    sprint::sprint93_config_from_example("soma_real_no_run_diagnostic_pass.toml", name)
}

#[test]
fn no_run_diagnostic_pass_is_deterministic_and_compile_only() {
    let runner = Sprint93TimeoutAttributionRunner::default();
    let cfg = config("real-no-run-diagnostic");
    let first = runner.run_real_no_run_diagnostic_pass(&cfg).expect("first");
    let second = runner
        .run_real_no_run_diagnostic_pass(&cfg)
        .expect("second");
    assert_eq!(first, second);
    assert_eq!(
        first.status,
        RealNoRunDiagnosticPassStatus::DiagnosticNoRunPassed
    );
    assert!(
        first
            .captured_targets
            .contains(&"krx_evidence_suite".to_string())
    );
    assert!(
        first
            .captured_targets
            .contains(&"dashboard_renderer_suite".to_string())
    );
}
