mod support;

use soma_zero::{RealNoRunGateAttemptV10Status, Sprint95CommitteeCliSafetyRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn real_no_run_gate_attempt_v10_is_not_run_by_default() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_real_no_run_gate_attempt_v10(&sprint::sprint95_config_from_example(
            "soma_real_no_run_gate_attempt_v10.toml",
            "real-no-run-gate-attempt-v10",
        ))
        .expect("report");
    assert_eq!(report.no_run_status, RealNoRunGateAttemptV10Status::NotRun);
}
