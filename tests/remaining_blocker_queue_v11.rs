mod support;

use soma_zero::{RemainingBlockerQueueV11Status, Sprint95CommitteeCliSafetyRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn remaining_blocker_queue_advances_to_baseline_signal() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_remaining_blocker_queue_v11(&sprint::sprint95_config_from_example(
            "soma_remaining_blocker_queue_v11.toml",
            "remaining-blocker-queue-v11",
        ))
        .expect("report");
    assert_eq!(
        report.queue_status,
        RemainingBlockerQueueV11Status::QueueAdvanced
    );
    assert_eq!(report.primary_next_family, "BaselineSignal");
    assert!(report.baseline_signal_entry_allowed);
}
