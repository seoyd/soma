mod support;

use soma_zero::{RemainingBlockerQueueV10Status, Sprint94DashboardRendererRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn remaining_blocker_queue_advances_to_committee_cli_safety() {
    let report = Sprint94DashboardRendererRecoveryRunner::default()
        .run_remaining_blocker_queue_v10(&sprint::sprint94_config_from_example(
            "soma_remaining_blocker_queue_v10.toml",
            "remaining-blocker-queue-v10",
        ))
        .expect("report");
    assert_eq!(
        report.queue_status,
        RemainingBlockerQueueV10Status::QueueAdvanced
    );
    assert_eq!(report.primary_next_family, "CommitteeCliSafety");
    assert!(report.committee_cli_safety_isolated);
}
