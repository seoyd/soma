mod support;

use soma_zero::{RealNoRunGateAttemptV4Status, Sprint89CandleRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn real_no_run_gate_attempt_v4_carries_forward_blocked_state_honestly() {
    let config = sprint::sprint89_config_from_example(
        "soma_real_no_run_gate_attempt_v4.toml",
        "real-no-run-v4",
    );
    let report = Sprint89CandleRecoveryRunner::default()
        .run_real_no_run_gate_attempt_v4(&config)
        .expect("report");
    assert_eq!(
        report.no_run_status,
        RealNoRunGateAttemptV4Status::RealNoRunStillBlocked
    );
    assert_eq!(report.blocked_families.len(), 6);
    assert!(!report.started);
}
