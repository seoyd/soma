mod support;

use soma_zero::{RealNoRunGateAttemptV3Status, Sprint88SevenBlockerRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn real_no_run_gate_attempt_v3_reports_blocked_truth_honestly() {
    let config = sprint::sprint88_config_from_example(
        "soma_real_no_run_gate_attempt_v3.toml",
        "real-no-run",
    );
    let report = Sprint88SevenBlockerRecoveryRunner::default()
        .run_real_no_run_gate_attempt_v3(&config)
        .expect("report");
    assert!(report.started);
    assert!(!report.finished);
    assert_eq!(
        report.no_run_status,
        RealNoRunGateAttemptV3Status::RealNoRunStillBlocked
    );
    assert_eq!(report.blocked_families.len(), 7);
}
