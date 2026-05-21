mod support;

use soma_zero::{
    CompileFamilyV2, RealNoRunGateAttemptV9Status, Sprint94DashboardRendererRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn real_no_run_gate_attempt_defaults_to_not_run() {
    let report = Sprint94DashboardRendererRecoveryRunner::default()
        .run_real_no_run_gate_attempt_v9(&sprint::sprint94_config_from_example(
            "soma_real_no_run_gate_attempt_v9.toml",
            "real-no-run-gate-attempt-v9",
        ))
        .expect("report");
    assert_eq!(report.no_run_status, RealNoRunGateAttemptV9Status::NotRun);
    assert_eq!(
        report.rerun_after_family,
        Some(CompileFamilyV2::DashboardRenderer)
    );
}
